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
}
