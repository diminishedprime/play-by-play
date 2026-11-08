## 1.0.0-alpha.27 (2026-11-08)

### Features

- serve each program a command runs in turn

## 1.0.0-alpha.26 (2026-11-08)

### Features

- show a live program's output in the viewer too

## 1.0.0-alpha.25 (2026-11-08)

### Features

- reflow a board's text to its follower's width

## 1.0.0-alpha.24 (2026-11-08)

### Features

- tell a program's board how many rows its viewer has

### Bug Fixes

- draw an indented link's indent outside the link

## 1.0.0-alpha.23 (2026-11-07)

### Bug Fixes

- keep spinners turning when their program redraws

## 1.0.0-alpha.22 (2026-11-07)

### Features

- paint red and gray by name

## 1.0.0-alpha.21 (2026-11-07)

### Features

- animate spinners on the live board

## 1.0.0-alpha.20 (2026-11-07)

### Bug Fixes

- write indents with every style off

## 1.0.0-alpha.19 (2026-11-07)

### Bug Fixes

- draw a board's rows in the order it yields them

## 1.0.0-alpha.18 (2026-11-07)

### Features

- indent nested lines and write through Painter without fmt::Write in scope

## 1.0.0-alpha.17 (2026-11-07)

### Features

- add the extension's native addon and move Log into the library
- nest markdown styles per line through a lazy Painter
- open a follower beside the viewer when a command claims it
- open a follower for every command that claims the viewer
- paint styled text through a nesting Painter
- print where the extension listens with socket --extension
- tell the extension when a command claims the viewer and keep logs in $TMPDIR/pbp

### Bug Fixes

- catch main and release up with origin before releasing

## 1.0.0-alpha.16 (2026-11-07)

### Bug Fixes

- give Claude play-by-play's context from a SessionStart command hook

## 1.0.0-alpha.15 (2026-11-06)

### Features

- connect a broadcast to a named socket, or open one offline

## 1.0.0-alpha.14 (2026-11-06)

### Features

- add a Play-by-Play: Start command that reopens the viewer
- cut a live view's lines by the columns their characters take
- cut a live view's lines by the columns they show, skipping escape codes
- keep watching when the viewer's theme can't be recorded
- run just check and just test as a pre-commit hook installed by just install-hooks
- say zsh isn't on the PATH when a command's shell can't start
- send a program's board to play-by-play run as protobuf lines and draw it there
- stop watch and follow cleanly when their terminal closes and measure them in just coverage

### Bug Fixes

- skip the pre-commit hook for release commits and format the release scripts in just fix

## 1.0.0-alpha.13 (2026-11-06)

### Bug Fixes

- tell Claude about play-by-play even when its session start runs late

## 1.0.0-alpha.12 (2026-11-06)

### Features

- give the Claude plugin its icon and keep the lab mirror in step with each release

## 1.0.0-alpha.11 (2026-11-06)

### Features

- clear a row its program drops from the live view
- draw a live view's lines at the viewer's width
- draw a live view's rows in the follower's terminal
- keep printing a live view's rows once its terminal goes
- let a program claim the viewer and follow its live view in its own terminal
- narrate each live view line to Claude only when it changes
- print a live view's changed rows as plain lines
- redraw a live view when its follower's width arrives

### Bug Fixes

- keep a command running when its program leaves before hearing the viewer's width
- keep a live program's command running when it disconnects before its follower's width
- keep one workspace version so merging main into alpha can't conflict on it
- keep the e2e world's file watches from missing changes
- let a live view's later row for a key overwrite the earlier one
- merge main into alpha over release version bumps and skip the push on dry runs
- read the e2e world's terminals without threads and close them in order
- record the viewer's theme by rename
- spawn the e2e world's programs without leaking descriptors
- tell Claude about play-by-play at session start instead of in every prompt's context

### Performance Improvements

- optimize dependencies in the dev profile

## 1.0.0-alpha.10 (2026-11-05)

### Features

- welcome the viewer when there's nothing to replay

## 1.0.0-alpha.9 (2026-11-04)

### Bug Fixes

- allow the guide skill's unwrap help command

## 1.0.0-alpha.8 (2026-11-04)

### Features

- leave color out of doctor outside a terminal

## 1.0.0-alpha.7 (2026-11-04)

### Bug Fixes

- run doctor skill with only allowed commands

## 1.0.0-alpha.6 (2026-11-04)

### Bug Fixes

- check unwrapped Bash commands as they are

## 1.0.0-alpha.5 (2026-11-04)

### Features

- add doctor command with guide and doctor skills

## 1.0.0-alpha.4 (2026-11-04)

### Bug Fixes

- show the viewer terminal when the window opens

## 1.0.0-alpha.3 (2026-11-04)

### Bug Fixes

- keep VS Code from restoring an old viewer terminal

## 1.0.0-alpha.2 (2026-11-04)

### Bug Fixes

- replace an existing play-by-play marketplace on install

## 1.0.0-alpha.1 (2026-11-04)

### Features

- check wrapped Bash commands as Claude wrote them
- close each note with its tool's name in its header's color
- color edit diffs with bat in the viewer's theme
- color markdown list items cyan
- color search links blue
- cycle header colors and show failed exits in red
- follow the workspace log live with play-by-play watch
- install the plugin through a local marketplace
- let a classic hook's verdict on a wrapped Bash command stand
- note each read from the plugin
- open the viewer from a VS Code extension
- record the viewer's width as it resizes
- release with semantic-release like bump-me
- render agent and fetch markdown to the viewer's width and theme
- render web search summaries as markdown
- run Bash commands through play-by-play in zsh
- scaffold play-by-play library, cli, and plugin
- show a read's header and file in the workspace log
- show every noted tool's header and result
- show reads through bat with numbered lines from where they start
- show written files through bat like reads
- tell Claude its output streams live to play-by-play
- unwrap a run wrapper back to Claude's command
- watch inside the workspace's tmux session
- wrap Claude's Bash commands as pbp -c

### Bug Fixes

- name the release package and use TypeScript 6 everywhere
- stream each write to the viewer with kqueue
