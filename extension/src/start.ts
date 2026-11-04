import type { Programs } from "#src/programs";
import type { Vscode } from "#src/vscode";

export async function start(vscode: Vscode, programs: Programs) {
  const [folder] = vscode.workspace.workspaceFolders ?? [];
  if (folder === undefined) {
    vscode.window.showErrorMessage(
      "play-by-play requires an open workspace with a folder.",
    );
    return;
  }
  const onPath = await programs.onPath("play-by-play");
  if (!onPath) {
    vscode.window.showErrorMessage(
      "play-by-play isn't on the PATH, so its viewer can't open.",
    );
    return;
  }
  vscode.window.createTerminal({
    name: "Play by Play",
    shellPath: "play-by-play",
    shellArgs: ["watch"],
    cwd: folder.uri.fsPath,
    location: vscode.TerminalLocation.Panel,
    isTransient: true,
  });
}
