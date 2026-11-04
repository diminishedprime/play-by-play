import { expect, test } from "vitest";
import { start } from "#src/start";
import { FakePrograms } from "#test/fakes/programs";
import { FakeVscode } from "#test/fakes/vscode";

test("opens the viewer in a terminal in the panel", async () => {
  const vscode = new FakeVscode(["/Users/me/programming/notes"]);

  await start(vscode, new FakePrograms(["play-by-play"]));

  expect(vscode.window.terminals).toEqual([
    {
      name: "Play by Play",
      shellPath: "play-by-play",
      shellArgs: ["watch"],
      cwd: "/Users/me/programming/notes",
      location: vscode.TerminalLocation.Panel,
    },
  ]);
});

test("opens no viewer without a folder", async () => {
  const vscode = new FakeVscode(undefined);

  await start(vscode, new FakePrograms(["play-by-play"]));

  expect(vscode.window.errors).toEqual([
    "play-by-play requires an open workspace with a folder.",
  ]);
  expect(vscode.window.terminals).toEqual([]);
});

test("watches from the workspace's folder", async () => {
  const vscode = new FakeVscode(["/Users/me/programming/recipes"]);

  await start(vscode, new FakePrograms(["play-by-play"]));

  expect(vscode.window.terminals).toEqual([
    expect.objectContaining({ cwd: "/Users/me/programming/recipes" }),
  ]);
});

test("says when play-by-play isn't on the PATH", async () => {
  const vscode = new FakeVscode(["/Users/me/programming/notes"]);

  await start(vscode, new FakePrograms([]));

  expect(vscode.window.errors).toEqual([
    "play-by-play isn't on the PATH, so its viewer can't open.",
  ]);
  expect(vscode.window.terminals).toEqual([]);
});
