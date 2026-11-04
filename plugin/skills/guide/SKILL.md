---
name: guide
description:
  Explain how play-by-play works and how to work alongside it. Use when the user
  asks what play-by-play is, how the viewer or the VS Code extension works, why
  their Bash commands look like `pbp -c '...'`, how it affects permissions or
  hooks, or why the viewer isn't showing.
---

# play-by-play

play-by-play shows every tool call Claude makes, live, in a terminal beside the
conversation. Explain the parts below that answer the user's question. Without a
specific question, give a short overview of each section.

## How it works

- **Bash commands.** The plugin rewrites each Bash command to
  `pbp -c '<command>'`. `pbp` runs the command in zsh and streams the command
  and its output to the workspace's log as they happen. Claude still gets the
  full output and the real exit code. A failed command's exit line shows in red.
- **Other tools.** Read, Write, Edit, WebFetch, WebSearch, and Agent are all
  formatted in a way that makes them nice to see.
- **The log.** Each workspace gets its own log under `$TMPDIR/play-by-play/`,
  named after the project folder.
- **The viewer.** `play-by-play watch` follows the workspace's log. It records
  the terminal's width and picks a code theme close to its background. The VS
  Code extension opens it as a "Play by Play" terminal in the panel when a
  folder opens. `play-by-play watch --tmux` runs it inside the workspace's tmux
  session instead.

## Working alongside it

- Keep the viewer open beside the conversation and watch long commands as they
  run. Step in as soon as something goes wrong rather than waiting for the turn
  to end.
- Claude is told not to filter command output through `tail`, `head`, or `grep`,
  because that cuts down what the user sees. Long output runs in the background
  so it streams in full, and Claude searches the saved output after.
- Expect more commands to run in the background than usual. Claude carries on or
  waits while they run, and their output still streams to the viewer.

## Permissions and hooks

- Permission rules are checked against the command Claude wrote, not the
  `pbp -c` wrapper, so existing allow and deny rules keep working.
- A custom approver, a classic `PreToolUse` hook on Bash, sees the wrapped
  command. If its verdict stands, play-by-play leaves it alone. The approver can
  recover the original command by piping the wrapped one through
  `play-by-play unwrap`.

  ```!
  if command -v play-by-play >/dev/null; then
    play-by-play unwrap --help
  else
    echo "play-by-play isn't on the PATH, so its unwrap help can't be shown. Run /play-by-play:doctor to find out why."
  fi
  ```

## Troubleshooting

When something isn't working, use the `play-by-play:doctor` skill. It checks the
setup and explains each fix.
