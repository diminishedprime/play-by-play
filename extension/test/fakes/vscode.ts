import {
  TerminalExitReason,
  type TerminalExitStatus,
  TerminalLocation,
  type TerminalOptions,
} from "vscode";
import {
  Vscode,
  type VSCodeCommands,
  type VSCodeTerminal,
  type VSCodeWindow,
} from "#src/vscode";

class FakeCommands implements VSCodeCommands {
  private readonly registered = new Map<string, () => unknown>();

  registerCommand(id: string, run: () => unknown) {
    this.registered.set(id, run);
    return {
      dispose: () => {
        this.registered.delete(id);
      },
    };
  }

  async run(id: string) {
    await this.registered.get(id)?.();
  }
}

class FakeWindow implements VSCodeWindow {
  readonly terminals: TerminalOptions[] = [];

  readonly errors: string[] = [];

  readonly shown: { preserveFocus: boolean | undefined }[] = [];

  private readonly opened: {
    -readonly [Key in keyof VSCodeTerminal]: VSCodeTerminal[Key];
  }[] = [];

  createTerminal(options: TerminalOptions) {
    this.terminals.push(options);
    const terminal = {
      show: (preserveFocus?: boolean) => {
        this.shown.push({ preserveFocus });
      },
      exitStatus: undefined as TerminalExitStatus | undefined,
    };
    this.opened.push(terminal);
    return terminal;
  }

  close(index: number) {
    const terminal = this.opened[index];
    if (terminal === undefined) {
      throw new Error(`no terminal ${String(index)} was opened`);
    }
    terminal.exitStatus = { code: undefined, reason: TerminalExitReason.User };
  }

  showErrorMessage(message: string) {
    this.errors.push(message);
  }
}

export class FakeVscode extends Vscode {
  declare readonly window: FakeWindow;

  declare readonly commands: FakeCommands;

  constructor(workspace: string[] | undefined) {
    super(
      new FakeWindow(),
      { workspaceFolders: workspace?.map((fsPath) => ({ uri: { fsPath } })) },
      { Panel: TerminalLocation.Panel },
      new FakeCommands(),
    );
  }
}
