import { expect, test } from "vitest";
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
