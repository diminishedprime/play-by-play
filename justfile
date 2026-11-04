set shell := ["bash", "-euo", "pipefail", "-c"]
set fallback

mod e2e

default:
    @just --list

# Apply formatting and fixes
fix: _deps
    cargo clippy --fix --allow-dirty --allow-staged --workspace --all-targets
    cargo fmt --all
    pnpm --dir plugin exec prettier --write --prose-wrap always --ignore-path ../.gitignore --log-level error .
    pnpm --dir extension exec eslint --fix .
    pnpm --dir extension exec prettier --write --prose-wrap always --ignore-path ../.gitignore --log-level error .

# Check formatting and lints
check: _deps
    parallel --tag --line-buffer --halt soon,fail=1 ::: \
        "cargo fmt --all --check" \
        "cargo clippy --workspace --all-targets -- -D warnings" \
        "pnpm --dir plugin exec prettier --check --prose-wrap always --ignore-path ../.gitignore ." \
        "pnpm --dir plugin exec tsc -p ." \
        "claude plugin validate --strict plugin" \
        "pnpm --dir extension exec prettier --check --prose-wrap always --ignore-path ../.gitignore ." \
        "pnpm --dir extension exec tsc --noEmit" \
        "pnpm --dir extension exec eslint ."

# Run all tests
test:
    cargo test --workspace --exclude play-by-play-e2e
    just e2e test
    claude plugin test plugin
    pnpm --dir extension exec vitest run

# Refresh Claude Code's types in plugin/.claude-plugin/types
plugin-types:
    CLAUDE_CODE_PLUGIN_DIR_WATCH=1 claude -p --plugin-dir plugin "Reply with ok." > /dev/null

# Install the play-by-play binary, the Claude plugin, and the VS Code extension
install: _deps
    cargo install --path cli --locked
    -claude plugin marketplace remove play-by-play
    claude plugin install play-by-play --marketplace "$PWD"
    pnpm --dir extension package
    code --install-extension extension/play-by-play-vscode-$(node -p 'require("./extension/package.json").version').vsix

# Remove the play-by-play binary, the Claude plugin, and the VS Code extension
uninstall:
    -cargo uninstall play-by-play-cli
    -claude plugin marketplace remove play-by-play
    -code --uninstall-extension diminishedprime.play-by-play-vscode

_deps:
    pnpm install --frozen-lockfile --loglevel error
    [ -d plugin/.claude-plugin/types ] || just plugin-types

# Prepare a release
[arg("dry_run", long="dry-run", value="--dry-run")]
[arg("alpha", long="alpha", value="--alpha")]
release dry_run="" alpha="": _deps
    node scripts/semantic-release.mts {{ dry_run }} {{ alpha }}
