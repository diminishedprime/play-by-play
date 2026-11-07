import { spawn } from "node:child_process";
import { createHash } from "node:crypto";
import { once } from "node:events";
import { mkdir, mkdtemp, realpath, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { basename, dirname, join } from "node:path";
import { expect, test, vi } from "vitest";
import { activate } from "#src/extension";
import { FakePrograms } from "#test/fakes/programs";
import { FakeVscode } from "#test/fakes/vscode";

test("the start command shows the viewer that's already open", async () => {
  const vscode = new FakeVscode(["/Users/me/programming/notes"]);

  await activate(vscode, new FakePrograms(["play-by-play"]), {
    subscriptions: [],
  });
  await vscode.commands.run("play-by-play.start");

  expect(vscode.window.terminals).toEqual([
    {
      name: "Play by Play",
      shellPath: "play-by-play",
      shellArgs: ["watch"],
      cwd: "/Users/me/programming/notes",
      location: vscode.TerminalLocation.Panel,
      isTransient: true,
    },
  ]);
  expect(vscode.window.shown).toEqual([
    { preserveFocus: true },
    { preserveFocus: true },
  ]);
});

test("the start command opens a new viewer once the last one closed", async () => {
  const vscode = new FakeVscode(["/Users/me/programming/notes"]);

  await activate(vscode, new FakePrograms(["play-by-play"]), {
    subscriptions: [],
  });
  vscode.window.close(0);
  await vscode.commands.run("play-by-play.start");

  const viewer = {
    name: "Play by Play",
    shellPath: "play-by-play",
    shellArgs: ["watch"],
    cwd: "/Users/me/programming/notes",
    location: vscode.TerminalLocation.Panel,
    isTransient: true,
  };
  expect(vscode.window.terminals).toEqual([viewer, viewer]);
});

test("the start command goes away when the extension's subscriptions are disposed", async () => {
  const vscode = new FakeVscode(["/Users/me/programming/notes"]);
  const context = { subscriptions: [] as { dispose(): unknown }[] };

  await activate(vscode, new FakePrograms(["play-by-play"]), context);
  vscode.window.close(0);
  for (const subscription of context.subscriptions) {
    subscription.dispose();
  }
  await vscode.commands.run("play-by-play.start");

  expect(vscode.window.terminals).toEqual([
    {
      name: "Play by Play",
      shellPath: "play-by-play",
      shellArgs: ["watch"],
      cwd: "/Users/me/programming/notes",
      location: vscode.TerminalLocation.Panel,
      isTransient: true,
    },
  ]);
});

test("opens a follower beside the viewer whenever a command claims it", async () => {
  await using workspace = await temporaryWorkspace();
  const first = "print 'a one' | live-view";
  const second = "print 'b two' | live-view";

  using extension = await activated(workspace.path);
  await viewerWatches(workspace.path);
  const firstPid = await claudeRuns(workspace.path, first);
  await vi.waitFor(() => {
    expect(extension.vscode.window.terminals).toContainEqual({
      name: first,
      shellPath: "play-by-play",
      shellArgs: ["follow", firstPid],
      cwd: workspace.path,
      location: { parentTerminal: extension.vscode.window.terminal(0) },
      isTransient: true,
    });
  });
  const secondPid = await claudeRuns(workspace.path, second);

  await vi.waitFor(() => {
    expect(extension.vscode.window.terminals).toContainEqual({
      name: second,
      shellPath: "play-by-play",
      shellArgs: ["follow", secondPid],
      cwd: workspace.path,
      location: { parentTerminal: extension.vscode.window.terminal(0) },
      isTransient: true,
    });
  });
  expect(extension.vscode.window.shown).toEqual([
    { preserveFocus: true },
    { preserveFocus: true },
    { preserveFocus: true },
  ]);
});

async function temporaryWorkspace() {
  const path = await realpath(await mkdtemp(join(tmpdir(), "workspace-")));
  return {
    path,
    [Symbol.asyncDispose]: async () => {
      await rm(path, { recursive: true, force: true });
    },
  };
}

async function activated(workspace: string) {
  const vscode = new FakeVscode([workspace]);
  const context = { subscriptions: [] as { dispose(): unknown }[] };
  await activate(vscode, new FakePrograms(["play-by-play"]), context);
  return {
    vscode,
    [Symbol.dispose]: () => {
      for (const subscription of context.subscriptions) {
        subscription.dispose();
      }
    },
  };
}

async function viewerWatches(workspace: string) {
  const folder = join(tmpdir(), "pbp");
  await mkdir(folder, { recursive: true });
  const hex = createHash("sha256")
    .update(workspace)
    .digest()
    .subarray(0, 6)
    .toString("hex");
  await writeFile(
    join(folder, `${basename(workspace)}_${hex}.viewer`),
    String(process.pid),
  );
}

async function claudeRuns(workspace: string, command: string) {
  const binary = process.env["PLAY_BY_PLAY_BIN"];
  if (binary === undefined) {
    throw new Error("PLAY_BY_PLAY_BIN names the play-by-play binary");
  }
  const pbp = spawn(binary, ["-c", command], {
    cwd: workspace,
    env: {
      ...process.env,
      CLAUDE_PROJECT_DIR: workspace,
      PATH: `${dirname(binary)}:${process.env["PATH"] ?? ""}`,
    },
    stdio: "ignore",
  });
  await once(pbp, "exit");
  return String(pbp.pid);
}
