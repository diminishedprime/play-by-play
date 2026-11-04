---
name: doctor
description:
  Check that play-by-play is set up and explain how to fix what isn't. Use when
  the viewer shows nothing, Claude's commands aren't wrapped in `pbp -c`, the VS
  Code extension can't find play-by-play, links don't open, or the user asks
  whether play-by-play is installed correctly.
---

# play-by-play doctor

```!
if command -v play-by-play >/dev/null; then
  play-by-play doctor
elif [ -x "$HOME/.cargo/bin/play-by-play" ]; then
  "$HOME/.cargo/bin/play-by-play" doctor
else
  echo "play-by-play isn't on the PATH or in ~/.cargo/bin, so doctor can't run."
fi
```

Relay each ❌ and ⚠️ line from doctor with its fix. Leave out the lines that
passed.

If the VS Code extension says play-by-play isn't on the PATH but doctor finds
it, VS Code was probably started from the Dock without `~/.cargo/bin` on its
`PATH`. Start VS Code from a terminal, or add `~/.cargo/bin` to the login
shell's `PATH`.

If doctor couldn't run, play-by-play isn't anywhere the skill looks. Ask the
user where they installed it, or help them install it with `just install` from
the play-by-play repo.
