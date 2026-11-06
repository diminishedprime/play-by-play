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
