use crate::world::World;

const INDENT: &str = "  ";
const OK: &str = "✅  \x1b[32m";
const FAILED: &str = "❌  \x1b[31m";
const WARNING: &str = "⚠️  \x1b[33m";
const SKIPPED: &str = "⏹️  \x1b[90m";
const END: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const NORMAL: &str = "\x1b[22m";
const PATH: &str = "\x1b[1m$PATH:\x1b[22m";
const LOG: &str = "\x1b[1mLog folder:\x1b[22m";
const TMUX: &str = "\x1b[1mtmux:\x1b[22m";
const PLUGIN: &str = "\x1b[1mPlugin:\x1b[22m";
const EXTENSION: &str = "\x1b[1mExtension:\x1b[22m";
const VERSION: &str = "\x1b[1mVersion:\x1b[22m";

#[tokio::test]
async fn passes_when_everything_is_set_up() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .succeeds()
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_cargos_bin_isnt_on_the_path() {
    let world = World::new();

    world.cargo_installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    let cargo_bin = world.cargo_bin().display().to_string();
    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{FAILED}{PATH} {BOLD}play-by-play{NORMAL} not on PATH
{INDENT}{INDENT}{INDENT}Fix: add {cargo_bin} to your login shell's PATH{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_play_by_play_isnt_on_the_path() {
    let world = World::new();

    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{FAILED}{PATH} {BOLD}play-by-play{NORMAL} not on PATH
{INDENT}{INDENT}{INDENT}Fix: run `just install` from the play-by-play repo{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_pbp_isnt_on_the_path() {
    let world = World::new();

    world.installs("play-by-play");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{FAILED}{PATH} {BOLD}pbp{NORMAL} not on PATH
{INDENT}{INDENT}{INDENT}Fix: run `just install` from the play-by-play repo{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_zsh_isnt_on_the_path() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{FAILED}{PATH} {BOLD}zsh{NORMAL} not on PATH
{INDENT}{INDENT}{INDENT}Fix: install zsh{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn skips_the_plugin_when_claude_isnt_on_the_path() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .succeeds()
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{SKIPPED}{PATH} {BOLD}claude{NORMAL} not on PATH, so the Claude Code plugin won't be checked{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn skips_vs_code_when_code_isnt_on_the_path() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .succeeds()
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{SKIPPED}{PATH} {BOLD}code{NORMAL} not on PATH, so the VS Code extension won't be checked{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn leaves_out_the_versions_when_theres_nothing_to_compare() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .succeeds()
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{SKIPPED}{PATH} {BOLD}claude{NORMAL} not on PATH, so the Claude Code plugin won't be checked{END}
{INDENT}{SKIPPED}{PATH} {BOLD}code{NORMAL} not on PATH, so the VS Code extension won't be checked{END}
{INDENT}{OK}{LOG} writable{END}
"
        ));
}

#[tokio::test]
async fn fails_when_the_log_folder_isnt_writable() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");
    world.log_folder_is_read_only();

    let log_folder = world.log_folder().display().to_string();
    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{FAILED}{LOG} unwritable
{INDENT}{INDENT}{INDENT}Fix: run `chmod u+w {log_folder}`{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn checks_tmuxs_hyperlinks_inside_tmux() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");
    world.tmux_reports_features(
        "terminal-features[0] xterm*:clipboard:focus:title\nterminal-features[1] *:hyperlinks",
    );

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .inside_tmux()
        .run()
        .succeeds()
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}tmux{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{TMUX} hyperlinks turned on{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn warns_when_tmuxs_hyperlinks_are_off() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");
    world.tmux_reports_features("terminal-features[0] xterm*:clipboard:focus:title");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .inside_tmux()
        .run()
        .succeeds()
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}tmux{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{WARNING}{TMUX} hyperlinks turned off. Viewer's links won't open
{INDENT}{INDENT}{INDENT}Fix: add `set -as terminal-features ',*:hyperlinks'` to your tmux config{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_the_plugin_isnt_installed() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_no_plugins();
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{FAILED}{PLUGIN} not installed
{INDENT}{INDENT}{INDENT}Fix: run `just install` from the play-by-play repo, then /reload-plugins{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_the_plugin_is_disabled() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", false);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{FAILED}{PLUGIN} disabled
{INDENT}{INDENT}{INDENT}Fix: run `claude plugin enable play-by-play@play-by-play`, then /reload-plugins{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_the_extension_isnt_installed() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.1.0", true);
    world.code_lists_extension("rust-lang.rust-analyzer@0.3.3105");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{FAILED}{EXTENSION} not installed
{INDENT}{INDENT}{INDENT}Fix: run `just install` from the play-by-play repo{END}
{INDENT}{OK}{VERSION} all versions match at 0.1.0{END}
"
        ));
}

#[tokio::test]
async fn fails_when_the_versions_differ() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.10.0", true);
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.1.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{FAILED}{VERSION} mismatch:
{INDENT}{INDENT}0.1.0  - play-by-play
{INDENT}{INDENT}0.10.0 - Claude plugin
{INDENT}{INDENT}0.1.0  - VS Code extension
{INDENT}{INDENT}{INDENT}Fix: run `just install` from the play-by-play repo{END}
"
        ));
}

#[tokio::test]
async fn fails_when_the_plugins_version_differs_without_vs_code() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.claude_lists_plugin("play-by-play@play-by-play", "0.2.0", true);

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}claude{NORMAL}{END}
{INDENT}{SKIPPED}{PATH} {BOLD}code{NORMAL} not on PATH, so the VS Code extension won't be checked{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{PLUGIN} installed and enabled{END}
{INDENT}{FAILED}{VERSION} mismatch:
{INDENT}{INDENT}0.1.0 - play-by-play
{INDENT}{INDENT}0.2.0 - Claude plugin
{INDENT}{INDENT}{INDENT}Fix: run `just install` from the play-by-play repo{END}
"
        ));
}

#[tokio::test]
async fn fails_when_the_extensions_version_differs_without_claude() {
    let world = World::new();

    world.installs("play-by-play");
    world.installs("pbp");
    world.has("zsh");
    world.code_lists_extension("diminishedprime.play-by-play-vscode@0.2.0");

    world
        .sut(["doctor"])
        .only_fakes_on_path()
        .run()
        .exits(1)
        .prints(&format!(
            r"Checking play-by-play's setup
{INDENT}{OK}{PATH} {BOLD}play-by-play{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}pbp{NORMAL}{END}
{INDENT}{OK}{PATH} {BOLD}zsh{NORMAL}{END}
{INDENT}{SKIPPED}{PATH} {BOLD}claude{NORMAL} not on PATH, so the Claude Code plugin won't be checked{END}
{INDENT}{OK}{PATH} {BOLD}code{NORMAL}{END}
{INDENT}{OK}{LOG} writable{END}
{INDENT}{OK}{EXTENSION} installed{END}
{INDENT}{FAILED}{VERSION} mismatch:
{INDENT}{INDENT}0.1.0 - play-by-play
{INDENT}{INDENT}0.2.0 - VS Code extension
{INDENT}{INDENT}{INDENT}Fix: run `just install` from the play-by-play repo{END}
"
        ));
}
