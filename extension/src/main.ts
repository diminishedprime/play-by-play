import { TerminalLocation, window, workspace } from "vscode";
import { Shell } from "#src/programs";
import { start } from "#src/start";
import { Vscode } from "#src/vscode";

export async function activate() {
  await start(new Vscode(window, workspace, TerminalLocation), new Shell());
}
