import { execFile } from "node:child_process";
import { once } from "node:events";
import { mkdtempDisposable, readFile, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import type { AddressInfo } from "node:net";
import { networkInterfaces, tmpdir } from "node:os";
import { join } from "node:path";
import { json } from "node:stream/consumers";
import { promisify } from "node:util";
import { format, resolveConfig } from "prettier";
import type { PrepareContext, PublishContext } from "semantic-release";
import { glob } from "tinyglobby";

const GIT_SUFFIX = /\.git$/;
const GITHUB_REPO = "diminishedprime/play-by-play";
const GH_REPO_FLAGS = ["--repo", GITHUB_REPO];
const LAB_MIRROR = "mirrored/play-by-play";
const LAB_CHECKOUT = "tree/lab/util/play-by-play";

const exec = promisify(execFile);

async function git(...args: string[]) {
  const { stdout } = await exec("git", args);
  return stdout.trim();
}

async function gh(...args: string[]) {
  const { stdout } = await exec("gh", args);
  return stdout.trim();
}

async function tea(...args: string[]) {
  const { stdout } = await exec("tea", args);
  return stdout.trim();
}

async function rewriteFile(path: string, gitea: string, github: string) {
  const contents = await readFile(path, "utf8");
  const rewritten = contents
    .replaceAll(`${gitea}/src/branch/main`, `${github}/blob/main`)
    .replaceAll(`${gitea}/raw/branch/main`, `${github}/raw/main`)
    .replaceAll(gitea, github);
  if (rewritten.includes(new URL(gitea).host)) {
    throw new Error(`Gitea references remain after the rewrite in ${path}`);
  }
  if (rewritten !== contents) {
    await writeFile(path, rewritten);
  }
}

async function rewriteForGithub(work: string, gitea: string, github: string) {
  const files = await glob("**", {
    cwd: work,
    dot: true,
    ignore: [".git"],
    absolute: true,
  });
  await Promise.all(files.map(async (path) => rewriteFile(path, gitea, github)));
}

async function existingRefs(...refs: string[]) {
  const output = await git("for-each-ref", "--format=%(refname)", ...refs);
  return new Set(output.split("\n"));
}

async function worktreeAdd(...args: string[]) {
  await git("worktree", "add", "--quiet", ...args);
}

async function addReleaseWorktree(work: string) {
  const refs = await existingRefs("refs/heads/release", "refs/remotes/origin/release");
  if (refs.has("refs/heads/release")) {
    await worktreeAdd(work, "release");
  } else if (refs.has("refs/remotes/origin/release")) {
    await worktreeAdd("-b", "release", work, "origin/release");
  } else {
    await worktreeAdd("--orphan", "-b", "release", work);
  }
  return {
    path: work,
    [Symbol.asyncDispose]: async () => {
      await git("worktree", "remove", "--force", work);
    },
  };
}

async function commitAndPush(work: string, version: string, tag: string) {
  await git(
    "-C",
    work,
    "commit",
    "--all",
    "--quiet",
    "--no-verify",
    "-m",
    `Play-by-Play ${version} release`,
  );
  await git("tag", tag, "release");
  await git("push", "--quiet", "origin", "release", tag);
}

async function publishToGithub(tag: string, notes: string, prerelease: boolean) {
  await git("push", "--quiet", "github", "release:main", tag);
  const flags = prerelease ? ["--prerelease"] : [];
  return gh("release", "create", tag, ...GH_REPO_FLAGS, "--title", tag, "--notes", notes, ...flags);
}

async function labMirrorClone() {
  const checkout = await git("rev-parse", "--show-toplevel");
  if (!checkout.endsWith(`/${LAB_CHECKOUT}`)) {
    throw new Error(`expected the checkout at <lab root>/${LAB_CHECKOUT}, it's at ${checkout}`);
  }
  const root = checkout.slice(0, -`/${LAB_CHECKOUT}`.length);
  return join(root, "library", LAB_MIRROR);
}

function labAddress() {
  const lab = Object.values(networkInterfaces())
    .flat()
    .find((info) => info?.family === "IPv4" && info.address.startsWith("10."));
  if (lab === undefined) {
    throw new Error("no 10.0.0.0/8 address for Gitea to deliver the mirror's webhook to");
  }
  return lab.address;
}

async function listenForTag(tag: string) {
  const server = createServer(async (request, response) => {
    const { ref } = (await json(request)) as { ref?: string };
    response.end();
    if (ref === tag || ref === `refs/tags/${tag}`) {
      server.emit("mirrored");
    }
  });
  server.listen(0);
  await once(server, "listening");
  return server;
}

async function addMirrorHook(url: string) {
  const config = { url, content_type: "json" };
  const body = { type: "gitea", active: true, events: ["create", "push"], config };
  const created = await tea("api", `/repos/${LAB_MIRROR}/hooks`, "-d", JSON.stringify(body));
  const { id } = JSON.parse(created) as { id: number };
  return {
    [Symbol.asyncDispose]: async () => {
      await tea("api", "-X", "DELETE", `/repos/${LAB_MIRROR}/hooks/${id}`);
    },
  };
}

async function syncLabMirror(tag: string) {
  await using server = await listenForTag(tag);
  const { port } = server.address() as AddressInfo;
  const address = labAddress();
  await using _hook = await addMirrorHook(`http://${address}:${port}/`);
  const mirrored = once(server, "mirrored", { signal: AbortSignal.timeout(30_000) });
  await tea("api", "-X", "POST", `/repos/${LAB_MIRROR}/mirror-sync`);
  await mirrored;
  await git("-C", await labMirrorClone(), "pull", "--ff-only", "--quiet");
}

async function setPluginVersion(cwd: string, version: string) {
  const path = join(cwd, "plugin", ".claude-plugin", "plugin.json");
  const manifest = JSON.parse(await readFile(path, "utf8")) as object;
  const formatted = await format(JSON.stringify({ ...manifest, version }), {
    ...(await resolveConfig(path)),
    filepath: path,
  });
  await writeFile(path, formatted);
}

export async function prepare(
  _pluginConfig: unknown,
  { cwd = process.cwd(), nextRelease }: PrepareContext,
) {
  const { version } = nextRelease;
  const changelog = join(cwd, "CHANGELOG.md");
  const options = await resolveConfig(changelog);
  const formatted = await format(await readFile(changelog, "utf8"), {
    ...options,
    filepath: changelog,
  });
  await writeFile(changelog, formatted);
  await exec("cargo", ["set-version", "--workspace", version], { cwd });
  const noGit = ["--no-git-tag-version", "--no-git-checks"];
  await exec("pnpm", ["--filter", "play-by-play-vscode", "version", version, ...noGit], { cwd });
  await setPluginVersion(cwd, version);
}

export async function publish(_pluginConfig: unknown, { nextRelease, branch }: PublishContext) {
  const { version, notes } = nextRelease;
  if (notes === undefined) {
    throw new Error("semantic-release generated no release notes");
  }
  const tag = `v${version}`;
  const origin = await git("remote", "get-url", "origin");
  const gitea = origin.replace(GIT_SUFFIX, "");
  const github = `https://github.com/${GITHUB_REPO}`;

  const tmp = join(tmpdir(), "play-by-play-release-");
  await using dir = await mkdtempDisposable(tmp);
  await using worktree = await addReleaseWorktree(dir.path);
  await git("-C", worktree.path, "read-tree", "-u", "--reset", branch.name);
  await rewriteForGithub(worktree.path, gitea, github);
  await commitAndPush(worktree.path, version, tag);
  const url = await publishToGithub(tag, notes, branch.prerelease !== undefined);
  await syncLabMirror(tag);
  return { name: tag, url };
}
