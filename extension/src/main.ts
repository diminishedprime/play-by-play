import {
  commands,
  type ExtensionContext,
  TerminalLocation,
  window,
  workspace,
} from "vscode";
import { activate as activateExtension } from "#src/extension";
import { Shell } from "#src/programs";
import { Vscode } from "#src/vscode";

export async function activate(context: ExtensionContext) {
  await activateExtension(
    new Vscode(window, workspace, TerminalLocation, commands),
    new Shell(),
    context,
  );
}
