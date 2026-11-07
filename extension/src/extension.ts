import { listen } from "#native";
import type { Programs } from "#src/programs";
import { start } from "#src/start";
import type { Vscode, VSCodeContext } from "#src/vscode";

export async function activate(
  vscode: Vscode,
  programs: Programs,
  context: VSCodeContext,
) {
  let viewer = await start(vscode, programs);
  context.subscriptions.push(
    vscode.commands.registerCommand("play-by-play.start", async () => {
      if (viewer === undefined || viewer.exitStatus !== undefined) {
        viewer = await start(vscode, programs);
      } else {
        viewer.show(true);
      }
    }),
  );
  const [folder] = vscode.workspace.workspaceFolders ?? [];
  if (folder === undefined) {
    return;
  }
  context.subscriptions.push(
    listen(folder.uri.fsPath, (claimed) => {
      if (viewer === undefined) {
        return;
      }
      vscode.window
        .createTerminal({
          name: claimed.command,
          shellPath: "play-by-play",
          shellArgs: ["follow", String(claimed.pid)],
          cwd: folder.uri.fsPath,
          location: { parentTerminal: viewer },
          isTransient: true,
        })
        .show(true);
    }),
  );
}
