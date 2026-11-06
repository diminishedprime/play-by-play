import { execFile } from "node:child_process";
import { mkdtempDisposable, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { Writable } from "node:stream";
import { promisify } from "node:util";
import semanticRelease from "semantic-release";

const exec = promisify(execFile);

async function git(...args: string[]) {
  const { stdout } = await exec("git", args);
  return stdout.trim();
}

async function alphaWorktree(work: string, dryRun: boolean) {
  await git("fetch", "--quiet", "origin");
  const remote = await git("for-each-ref", "--format=%(refname)", "refs/remotes/origin/alpha");
  const start = remote === "" ? "main" : "origin/alpha";
  await git("worktree", "add", "--quiet", "-B", "alpha", work, start);
  await git("-C", work, "merge", "--quiet", "--no-edit", "-X", "theirs", "main");
  await symlink(resolve("node_modules"), join(work, "node_modules"));
  if (!dryRun) {
    await git("-C", work, "push", "--quiet", "origin", "alpha");
  }
  return {
    path: work,
    [Symbol.asyncDispose]: async () => {
      await git("worktree", "remove", "--force", work);
    },
  };
}

async function release(dryRun: boolean, cwd: string) {
  const result = await semanticRelease(
    { ci: false, dryRun },
    {
      cwd,
      env: {
        ...process.env,
        GIT_CONFIG_COUNT: "1",
        GIT_CONFIG_KEY_0: "core.hooksPath",
        GIT_CONFIG_VALUE_0: "/dev/null",
      },
      stdout: new Writable({
        write(_chunk, _encoding, done) {
          done();
        },
      }) as unknown as NodeJS.WriteStream,
      stderr: process.stderr,
    },
  );

  if (result === false) {
    console.log("No relevant changes, nothing to release.");
    return;
  }

  const {
    nextRelease: { version, type, notes },
    releases,
  } = result;

  if (dryRun) {
    console.log(`Would release ${version} (${type})`);
  } else {
    console.log(`Released ${version} (${type})`);
  }
  for (const { url } of releases) {
    if (url !== undefined) {
      console.log(url);
    }
  }
  if (notes !== undefined) {
    console.log(`\n${notes}`);
  }
}

const dryRun = process.argv.includes("--dry-run");

if (process.argv.includes("--alpha")) {
  await using dir = await mkdtempDisposable(join(tmpdir(), "play-by-play-alpha-"));
  await using worktree = await alphaWorktree(join(dir.path, "alpha"), dryRun);
  await release(dryRun, worktree.path);
} else {
  await release(dryRun, process.cwd());
}
