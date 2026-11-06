import type * as vscode from "vscode";

export type VSCodeTerminal = Pick<vscode.Terminal, "show" | "exitStatus">;

export interface VSCodeWindow {
  createTerminal(options: vscode.TerminalOptions): VSCodeTerminal;
  showErrorMessage(message: string): unknown;
}

export interface VSCodeWorkspace {
  readonly workspaceFolders:
    readonly { readonly uri: Pick<vscode.Uri, "fsPath"> }[] | undefined;
}

export type VSCodeTerminalLocation = Pick<
  typeof vscode.TerminalLocation,
  "Panel"
>;

export interface VSCodeCommands {
  registerCommand(id: string, run: () => unknown): vscode.Disposable;
}

export type VSCodeContext = Pick<vscode.ExtensionContext, "subscriptions">;

export class Vscode {
  constructor(
    readonly window: VSCodeWindow,
    readonly workspace: VSCodeWorkspace,
    readonly TerminalLocation: VSCodeTerminalLocation,
    readonly commands: VSCodeCommands,
  ) {}
}
