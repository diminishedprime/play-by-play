import type { Programs } from "#src/programs";

export class FakePrograms implements Programs {
  constructor(private readonly installed: string[]) {}

  async onPath(name: string) {
    return Promise.resolve(this.installed.includes(name));
  }
}
