import { execFile } from "node:child_process";
import { promisify } from "node:util";

const run = promisify(execFile);

export interface Programs {
  onPath(name: string): Promise<boolean>;
}

export class Shell implements Programs {
  async onPath(name: string) {
    try {
      await run("sh", ["-c", 'command -v "$1"', "sh", name]);
      return true;
    } catch {
      return false;
    }
  }
}
