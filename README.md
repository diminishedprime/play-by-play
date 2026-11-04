# Play by Play

Shows every tool call Claude Code makes live, in a VS Code terminal beside the
conversation.

## Install

You need:

- [Homebrew]
- [Rust]
- [pnpm]
- [Claude Code]
- [VS Code] with `code` on your `PATH`

To install from scratch, clone the repo and run the bootstrap script. It checks
the prerequisites, installs [just] with Homebrew if missing, then installs the
`play-by-play` command, the Claude Code plugin, and the VS Code extension.

```sh
git clone https://github.com/diminishedprime/play-by-play.git
cd play-by-play
zsh scripts/bootstrap.zsh
```

To reinstall later, run:

```sh
just install
```

[Homebrew]: https://brew.sh
[Rust]: https://rustup.rs
[pnpm]: https://pnpm.io/installation
[just]: https://just.systems
[Claude Code]: https://claude.com/claude-code
[VS Code]: https://code.visualstudio.com
