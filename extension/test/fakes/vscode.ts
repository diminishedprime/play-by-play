import { TerminalLocation, type TerminalOptions } from "vscode";
import { Vscode, type VSCodeWindow } from "#src/vscode";

class FakeWindow implements VSCodeWindow {
  readonly terminals: TerminalOptions[] = [];

  readonly errors: string[] = [];

  createTerminal(options: TerminalOptions) {
    this.terminals.push(options);
  }

  showErrorMessage(message: string) {
    this.errors.push(message);
  }
}

export class FakeVscode extends Vscode {
  declare readonly window: FakeWindow;

  constructor(workspace: string[] | undefined) {
    super(
      new FakeWindow(),
      { workspaceFolders: workspace?.map((fsPath) => ({ uri: { fsPath } })) },
      { Panel: TerminalLocation.Panel },
    );
  }
}
