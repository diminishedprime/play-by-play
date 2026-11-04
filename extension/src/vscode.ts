import type * as vscode from "vscode";

export interface VSCodeWindow {
  createTerminal(options: vscode.TerminalOptions): unknown;
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

export class Vscode {
  constructor(
    readonly window: VSCodeWindow,
    readonly workspace: VSCodeWorkspace,
    readonly TerminalLocation: VSCodeTerminalLocation,
  ) {}
}
