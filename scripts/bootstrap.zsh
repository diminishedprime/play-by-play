#!/usr/bin/env zsh

set -euo pipefail

cd "${0:A:h}/.."

missing=()
for tool in brew cargo pnpm code claude; do
	if ((!$+commands[$tool])); then
		missing+=("$tool")
	fi
done

if ((${#missing} > 0)); then
	print -u2 -r -- "Play by Play needs ${(j:, :)missing}. See Install in the README."
	exit 1
fi

if ((!$+commands[just])); then
	brew install just
fi
just install
