//! Runs the `play-by-play` binary named by `PLAY_BY_PLAY_BIN`, which
//! `just e2e test` builds and sets.

mod doctor;
mod world;

use std::fs;
use std::path::Path;

use serde_json::json;

use world::{Style, Viewer, World};

const BOLD: &str = "\x1b[1m";
const NORMAL: &str = "\x1b[22m";
const ITALIC: &str = "\x1b[3m";
const UPRIGHT: &str = "\x1b[23m";
const UNDERLINE: &str = "\x1b[4m";
const NO_UNDERLINE: &str = "\x1b[24m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const DEFAULT_COLOR: &str = "\x1b[39m";
const LINK_TO_DOCS: &str = "\x1b]8;;https://example.com/docs\x1b\\";
const LINK_END: &str = "\x1b]8;;\x1b\\";

#[tokio::test]
async fn reports_its_version_for_the_plugins_install_check() {
    let world = World::new();

    world
        .sut(["--version"])
        .run()
        .await
        .succeeds()
        .prints("play-by-play 0.1.0\n");
    world.close().await;
}

#[tokio::test]
async fn numbers_a_partial_read_from_its_first_line() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world
        .claude()
        .reads_from("/notes.md", "forty\nforty-one\n", 40)
        .await
        .succeeds();

    let top = format!("─────┬{}", "─".repeat(71));
    let bottom = format!("─────┴{}", "─".repeat(71));
    world
        .viewer()
        .sees(&[
            "read: /notes.md",
            &top,
            "  40 │ forty",
            "  41 │ forty-one",
            &bottom,
            "read: end",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_the_viewer_the_file_claude_wrote() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer_theme_becomes("Solarized (dark)").await;
    world
        .claude()
        .writes("/main.rs", "fn main() {\n    let x = 1;\n}\n")
        .await
        .succeeds();

    let top = format!("─────┬{}", "─".repeat(71));
    let bottom = format!("─────┴{}", "─".repeat(71));
    world
        .viewer()
        .sees(&[
            "write: /main.rs",
            &top,
            "   1 │ fn main() {",
            "   2 │     let x = 1;",
            "   3 │ }",
            &bottom,
            "write: end",
        ])
        .await;
    assert_eq!(world.viewer().style_of("let"), Some(Style::SolarizedBlue));
    world.close().await;
}

#[tokio::test]
async fn colors_an_edits_diff() {
    let mut world = World::new();

    world.viewer_theme_becomes("Solarized (dark)").await;
    world
        .claude()
        .edits(
            "/main.rs",
            &[" fn main() {", "-    let x = 1;", "+    let x = 2;", " }"],
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "edit: /main.rs",
            "@@ -1,3 +1,3 @@",
            " fn main() {",
            "-    let x = 1;",
            "+    let x = 2;",
            " }",
            "edit: end",
        ])
        .await;
    assert_eq!(
        world.viewer().style_of("    let x = 1;"),
        Some(Style::SolarizedRed)
    );
    assert_eq!(
        world.viewer().style_of("    let x = 2;"),
        Some(Style::SolarizedGreen)
    );
    world.close().await;
}

#[tokio::test]
async fn unwraps_a_wrapped_command_for_cc_toolgate() {
    let world = World::new();

    world
        .sut(["unwrap"])
        .stdin("pbp -c 'echo '\\''hi'\\'''")
        .run()
        .await
        .succeeds()
        .prints("echo 'hi'");
    world.close().await;
}

#[tokio::test]
async fn leaves_a_command_it_didnt_wrap_as_it_is() {
    let world = World::new();

    world
        .sut(["unwrap"])
        .stdin("ls")
        .run()
        .await
        .succeeds()
        .prints("ls");
    world.close().await;
}

#[tokio::test]
async fn gives_claude_play_by_plays_context_when_a_session_starts() {
    let world = World::unwatched();

    world
        .sut(["hook", "session-start"])
        .run()
        .await
        .succeeds()
        .prints(
            &json!({
                "hookSpecificOutput": {
                    "hookEventName": "SessionStart",
                    "additionalContext": include_str!("../../../cli/src/play-by-play.md"),
                },
            })
            .to_string(),
        );
    world.close().await;
}

#[tokio::test]
async fn watches_in_the_workspaces_tmux_session() {
    let world = World::new();

    world
        .sut(["watch", "--tmux"])
        .project(Path::new("/Users/me/programming/notes"))
        .run()
        .await
        .succeeds();

    world.tmux_ran(&[
        "new-session",
        "-A",
        "-s",
        "notes_75326a409771",
        "tmux set-option destroy-unattached off && play-by-play watch",
    ]);
    world.close().await;
}

#[tokio::test]
async fn shows_the_viewer_the_file_claude_read() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer_theme_becomes("Solarized (dark)").await;
    world
        .claude()
        .reads("/main.rs", "fn main() {\n    let x = 1;\n}\n")
        .await
        .succeeds();

    let top = format!("─────┬{}", "─".repeat(71));
    let bottom = format!("─────┴{}", "─".repeat(71));
    world
        .viewer()
        .sees(&[
            "read: /main.rs",
            &top,
            "   1 │ fn main() {",
            "   2 │     let x = 1;",
            "   3 │ }",
            &bottom,
            "read: end",
        ])
        .await;
    assert_eq!(world.viewer().style_of("let"), Some(Style::SolarizedBlue));
    assert_eq!(world.viewer().style_of("read: end"), Some(Style::Green));
    world.close().await;
}

#[tokio::test]
async fn fails_a_command_when_zsh_isnt_on_the_path() {
    let world = World::unwatched();

    world
        .sut(["-c", "echo hi"])
        .only_fakes_on_path()
        .run()
        .await
        .exits(1)
        .prints("")
        .prints_error("play-by-play: zsh isn't on the PATH, so the command can't run\n");
    world.close().await;
}

#[tokio::test]
async fn runs_a_command_through_pbp() {
    let world = World::unwatched();

    world.installs("pbp");

    world
        .claude()
        .runs("pbp -c 'echo hi'")
        .await
        .succeeds()
        .prints("hi\n");
    world.close().await;
}

#[tokio::test]
async fn shows_the_viewer_a_command_claude_ran() {
    let mut world = World::new();

    world
        .claude()
        .runs("echo out; exit 3")
        .await
        .exits(3)
        .prints("out\n")
        .prints_error("");

    world
        .viewer()
        .sees(&["$ echo out; exit 3", "out", "exit: 3"])
        .await;
    world.close().await;
}

#[tokio::test]
async fn formats_an_agents_markdown() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer_theme_becomes("Solarized (dark)").await;
    world.claude().asks_agent(
        "find **everything**",
        r"## Overview of `notify` crate

The viewer follows the log with `kqueue`, so take **care**: every write reaches it straight away and nothing waits for the command to finish.

See *these* [docs](https://example.com/docs) and keep **bold _both_**.

- first item
- a second item that is long enough to wrap past the viewer's seventy-seven columns and keep going on its way
- see [the docs](https://example.com/docs) here

```rust
let color = log.next_color()?;
```

```
plain code
```
",
    )
    .await
    .succeeds();

    world
        .viewer()
        .sees(&[
            "agent: explore",
            "prompt:",
            "find everything",
            "response:",
            "Overview of notify crate",
            "",
            "The viewer follows the log with kqueue, so take care: every write reaches it",
            "straight away and nothing waits for the command to finish.",
            "",
            "See these docs and keep bold both.",
            "",
            "- first item",
            "- a second item that is long enough to wrap past the viewer's seventy-seven",
            "  columns and keep going on its way",
            "- see the docs here",
            "",
            "let color = log.next_color()?;",
            "",
            "plain code",
            "agent: end",
        ])
        .await;
    assert_eq!(world.viewer().style_of("everything"), Some(Style::Bold));
    assert_eq!(world.viewer().style_of("- "), Some(Style::Cyan));
    assert_eq!(world.viewer().style_of("Overview"), Some(Style::Blue));
    assert_eq!(world.viewer().style_of("kqueue"), Some(Style::Green));
    assert_eq!(world.viewer().style_of("care"), Some(Style::Bold));
    assert_eq!(world.viewer().style_of("let"), Some(Style::SolarizedBlue));
    for painted in [
        format!("{BOLD}everything{NORMAL}"),
        format!("{CYAN}- first item{DEFAULT_COLOR}"),
        format!(
            "seventy-seven\r\n{DEFAULT_COLOR}  {CYAN}columns and keep going on its way{DEFAULT_COLOR}"
        ),
        format!("{BOLD}{BLUE}Overview of {GREEN}notify{BLUE} crate{NORMAL}{DEFAULT_COLOR}"),
        format!("{GREEN}kqueue{DEFAULT_COLOR}"),
        format!("{BOLD}care{NORMAL}:"),
        format!("{ITALIC}these{UPRIGHT}"),
        format!(
            "{LINK_TO_DOCS}{UNDERLINE}{BLUE}docs{NO_UNDERLINE}{DEFAULT_COLOR}{LINK_END} and keep {BOLD}bold {ITALIC}both{NORMAL}{UPRIGHT}."
        ),
        format!(
            "{CYAN}- see {LINK_TO_DOCS}{UNDERLINE}{BLUE}the docs{NO_UNDERLINE}{CYAN}{LINK_END} here{DEFAULT_COLOR}"
        ),
    ] {
        assert!(
            world.viewer().printed(&painted),
            "the viewer was never sent {painted:?}"
        );
    }
    world.close().await;
}

#[tokio::test]
async fn shows_a_read_images_size() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "read",
            &json!({ "file_path": "/cat.png" }),
            &json!({
                "type": "image",
                "file": { "base64": "", "type": "image/png", "originalSize": 2048 },
            }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&["read: /cat.png", "an image, 2048 bytes", "read: end"])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_a_read_notebooks_cell_count() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "read",
            &json!({ "file_path": "/plot.ipynb" }),
            &json!({
                "type": "notebook",
                "file": { "filePath": "/plot.ipynb", "cells": [{}, {}, {}] },
            }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&["read: /plot.ipynb", "a notebook, 3 cells", "read: end"])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_a_read_pdfs_size() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "read",
            &json!({ "file_path": "/paper.pdf" }),
            &json!({
                "type": "pdf",
                "file": { "filePath": "/paper.pdf", "base64": "", "originalSize": 4096 },
            }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&["read: /paper.pdf", "a PDF, 4096 bytes", "read: end"])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_where_a_pdf_read_as_pages_went() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "read",
            &json!({ "file_path": "/paper.pdf", "pages": "1-3" }),
            &json!({
                "type": "parts",
                "file": {
                    "filePath": "/paper.pdf",
                    "originalSize": 4096,
                    "count": 3,
                    "outputDir": "/tmp/pages",
                },
            }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "read: /paper.pdf",
            "a PDF, as 3 page images in /tmp/pages",
            "read: end",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn says_a_reread_file_is_unchanged() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "read",
            &json!({ "file_path": "/notes.md" }),
            &json!({ "type": "file_unchanged", "file": { "filePath": "/notes.md" } }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "read: /notes.md",
            "unchanged since it was last read",
            "read: end",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_where_a_background_agent_writes() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "agent",
            &json!({ "description": "explore", "prompt": "look around" }),
            &json!({
                "status": "async_launched",
                "agentId": "a1",
                "description": "explore",
                "prompt": "look around",
                "outputFile": "/tmp/a1.output",
            }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "agent: explore",
            "prompt:",
            "look around",
            "running in the background: /tmp/a1.output",
            "agent: end",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_where_a_cloud_agent_runs() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "agent",
            &json!({ "description": "explore", "prompt": "look around" }),
            &json!({
                "status": "remote_launched",
                "taskId": "t1",
                "sessionUrl": "https://claude.ai/code/t1",
                "description": "explore",
                "prompt": "look around",
                "outputFile": "/tmp/t1.output",
            }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "agent: explore",
            "prompt:",
            "look around",
            "running in the cloud: https://claude.ai/code/t1",
            "agent: end",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_a_result_of_an_unknown_kind_as_it_came() {
    let mut world = World::new();

    world
        .claude()
        .noted(
            "agent",
            &json!({ "description": "explore", "prompt": "look around" }),
            &json!({ "status": "teleported" }),
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "agent: explore",
            "prompt:",
            "look around",
            r#"{"status":"teleported"}"#,
            "agent: end",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn joins_a_fetched_pages_broken_lines_and_emphasis() {
    let mut world = World::new();

    world
        .claude()
        .fetches(
            "https://example.com",
            "what does it say",
            "It says\nto take *care*.",
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "fetch: https://example.com",
            "prompt: what does it say",
            "It says to take care.",
            "fetch: end",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn formats_a_fetched_pages_markdown() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world
        .claude()
        .fetches(
            "https://example.com",
            "what does it say",
            "It says to take **care**.

Before you ship anything, run every test you have and then build with `cargo build --release` first.

- first paragraph

  second paragraph
- next item

Then:

- outer
  - inner one
  - inner two
- after",
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "fetch: https://example.com",
            "prompt: what does it say",
            "It says to take care.",
            "",
            "Before you ship anything, run every test you have and then build with",
            "cargo build --release first.",
            "",
            "- first paragraph",
            "",
            "  second paragraph",
            "- next item",
            "",
            "Then:",
            "",
            "- outer",
            "  - inner one",
            "  - inner two",
            "- after",
            "fetch: end",
        ])
        .await;
    assert_eq!(world.viewer().style_of("care"), Some(Style::Bold));
    let painted = format!("{GREEN}cargo build --release{DEFAULT_COLOR} first.");
    assert!(
        world.viewer().printed(&painted),
        "the viewer was never sent {painted:?}"
    );
    world.close().await;
}

#[tokio::test]
async fn formats_a_searchs_summary_markdown() {
    let mut world = World::new();

    world
        .claude()
        .searches(
            "file watching",
            &[("notify crate docs", "https://docs.rs/notify")],
            "Use **kqueue** to watch files.",
        )
        .await
        .succeeds();

    world
        .viewer()
        .sees(&[
            "search: file watching",
            "- notify crate docs",
            "Use kqueue to watch files.",
            "search: end",
        ])
        .await;
    assert_eq!(world.viewer().style_of("kqueue"), Some(Style::Bold));
    assert_eq!(
        world.viewer().style_of("notify crate docs"),
        Some(Style::Blue)
    );
    world.close().await;
}

#[tokio::test]
async fn picks_the_code_theme_closest_to_the_viewers_background() {
    let world = World::new();

    world.viewer_theme_becomes("Solarized (dark)").await;
    world.close().await;
}

#[tokio::test]
async fn keeps_watching_when_it_cant_record_its_theme() {
    let mut world = World::unwatched();

    fs::create_dir(world.log_folder().join("notes_75326a409771.theme"))
        .expect("the theme's path is taken");
    world.reopen_viewer_for(Path::new("/Users/me/programming/notes"));

    world
        .viewer()
        .sees(&[
            "play-by-play is watching this workspace.",
            "Claude's commands show up here as they run.",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn welcomes_the_viewer_when_theres_nothing_to_replay() {
    let mut world = World::new();

    world
        .viewer()
        .sees(&[
            "play-by-play is watching this workspace.",
            "Claude's commands show up here as they run.",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn replays_the_last_command_instead_of_the_welcome() {
    let mut world = World::new();

    world.claude().runs("echo hi").await.succeeds();
    world.viewer().sees(&["$ echo hi", "hi", "exit: 0"]).await;
    world.reopen_viewer();
    world.viewer().sees(&["$ echo hi", "hi", "exit: 0"]).await;

    assert!(
        !world
            .viewer()
            .printed("play-by-play is watching this workspace.")
    );
    world.close().await;
}

#[tokio::test]
async fn records_the_viewers_width() {
    let world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.close().await;
}

#[tokio::test]
async fn records_the_viewers_width_when_it_resizes() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer().resize(100);

    world.viewer_width_becomes(100).await;
    world.close().await;
}

#[tokio::test]
async fn gives_each_tool_call_the_next_color() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world
        .claude()
        .reads("/notes.md", "first\n")
        .await
        .succeeds();
    let top = format!("─────┬{}", "─".repeat(71));
    let bottom = format!("─────┴{}", "─".repeat(71));
    world
        .viewer()
        .sees(&[
            "read: /notes.md",
            &top,
            "   1 │ first",
            &bottom,
            "read: end",
        ])
        .await;
    world.claude().runs("echo hi").await.succeeds();
    world.viewer().sees(&["$ echo hi", "hi", "exit: 0"]).await;

    assert_eq!(world.viewer().colors(), [Style::Green, Style::Yellow]);
    world.close().await;
}

#[tokio::test]
async fn cycles_back_to_the_first_color_after_the_last() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.claude().runs("echo 1").await.succeeds();
    world.claude().runs("echo 2").await.succeeds();
    world.claude().runs("echo 3").await.succeeds();
    world
        .claude()
        .reads("/notes.md", "fourth\n")
        .await
        .succeeds();
    world.claude().runs("echo 5").await.succeeds();
    world.claude().runs("echo 6").await.succeeds();
    world.viewer().sees(&["$ echo 6", "6", "exit: 0"]).await;

    assert_eq!(
        world.viewer().colors(),
        [
            Style::Green,
            Style::Yellow,
            Style::Blue,
            Style::Magenta,
            Style::Cyan,
            Style::Green,
        ]
    );
    world.close().await;
}

#[tokio::test]
async fn shows_a_failed_commands_exit_in_red() {
    let mut world = World::new();

    world.claude().runs("exit 3").await.exits(3);
    world.viewer().sees(&["$ exit 3", "exit: 3"]).await;

    assert_eq!(world.viewer().exit_colors(), [Style::Red]);
    world.close().await;
}

#[tokio::test]
async fn shows_a_successful_commands_exit_in_its_headers_color() {
    let mut world = World::new();

    world.claude().runs("true").await.succeeds();
    world.viewer().sees(&["$ true", "exit: 0"]).await;

    assert_eq!(world.viewer().exit_colors(), [Style::Green]);
    world.close().await;
}

#[tokio::test]
async fn offers_a_command_a_socket_while_a_viewer_watches() {
    let world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;

    world
        .claude()
        .runs(r#"test -S "$PLAY_BY_PLAY_SOCKET""#)
        .await
        .succeeds();
    world.close().await;
}

#[tokio::test]
async fn tells_the_extension_which_command_claimed_the_viewer() {
    let world = World::new();
    let command =
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && print 'a one' | pbp-test downloads"#;

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let extension = world.extension_listens();
    world.claude().runs(command).await.succeeds();
    let pid = world.workspace_file("pid").await;

    extension.heard_claimed(pid.trim(), command).await;
    world.close().await;
}

#[tokio::test]
async fn offers_no_socket_while_nobody_watches() {
    let world = World::unwatched();

    world
        .claude()
        .runs(r#"test -z "$PLAY_BY_PLAY_SOCKET""#)
        .await
        .succeeds();
    world.close().await;
}

#[tokio::test]
async fn stops_watching_when_its_terminal_closes() {
    let mut world = World::new();

    world
        .viewer()
        .sees(&[
            "play-by-play is watching this workspace.",
            "Claude's commands show up here as they run.",
        ])
        .await;
    let exited = world.viewer_hangs_up().await;

    assert_eq!(exited.code(), Some(0));
    world.close().await;
}

#[tokio::test]
async fn stops_following_when_its_terminal_closes() {
    let mut world = World::new();

    world.follow("4242");
    world
        .follower_width_becomes("4242", Viewer::FOLLOWER_COLUMNS)
        .await;
    let exited = world.follower_hangs_up().await;

    assert_eq!(exited.code(), Some(0));
    world.close().await;
}

#[tokio::test]
async fn offers_no_socket_once_the_viewer_has_gone() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.close_viewer().await;

    world
        .claude()
        .runs(r#"test -z "$PLAY_BY_PLAY_SOCKET""#)
        .await
        .succeeds();
    world.close().await;
}

#[tokio::test]
async fn shows_a_live_programs_lines_in_the_viewer() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world
        .claude()
        .runs("print 'a one' | pbp-test downloads")
        .await
        .succeeds()
        .prints("downloading a\n");

    world
        .viewer()
        .sees(&[
            "$ print 'a one' | pbp-test downloads",
            "downloading a",
            "exit: 0",
        ])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_the_viewer_a_command_while_it_runs() {
    let mut world = World::new();

    let running = world.claude().starts("echo started; read; echo finished");

    world
        .viewer()
        .sees(&["$ echo started; read; echo finished", "started"])
        .await;
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn shows_the_viewer_a_commands_errors_and_keeps_them_on_stderr() {
    let mut world = World::new();

    world
        .claude()
        .runs("echo err >&2")
        .await
        .exits(0)
        .prints("")
        .prints_error("err\n");

    world
        .viewer()
        .sees(&["$ echo err >&2", "err", "exit: 0"])
        .await;
    world.close().await;
}

#[tokio::test]
async fn narrates_to_claude_while_nobody_watches() {
    let world = World::unwatched();

    world.installs("pbp-test");

    world
        .claude()
        .runs(r"printf 'a one\nb two\n' | pbp-test downloads")
        .await
        .succeeds()
        .prints("downloading a\ndownloading b\n");
    world.close().await;
}

#[tokio::test]
async fn narrates_each_download_once_however_often_it_progresses() {
    let world = World::unwatched();

    world.installs("pbp-test");

    world
        .claude()
        .runs("print -l 'a 1%' 'a 10%' 'a 95%' 'a 100%' 'a' | pbp-test downloads")
        .await
        .succeeds()
        .prints("downloading a\ndownloaded a\n");
    world.close().await;
}

#[tokio::test]
async fn follows_a_program_that_claims_the_viewer_after_its_follower_opens() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && read -r; print 'a one' | pbp-test downloads"#,
    );
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world
        .follower_width_becomes(pid.trim(), Viewer::FOLLOWER_COLUMNS)
        .await;
    running.finishes().await.succeeds();
    world.follower().sees(&["a one"]).await;
    world.close().await;
}

#[tokio::test]
async fn indents_a_programs_link_outside_of_the_link() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && { print 'a 100%'; read -r } | pbp-test downloads"#,
    );
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world.follower().sees(&["a 100%", "  open"]).await;
    assert!(
        world
            .follower()
            .printed("  \x1b]8;;file:///downloads/a\x1b\\"),
        "the indent was drawn inside the link"
    );
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn paints_a_programs_diff_in_color() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && { print 'a one->two'; read -r } | pbp-test downloads"#,
    );
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world
        .follower()
        .sees(&["a", "  @@ -1 +1 @@", "  -one", "  +two"])
        .await;
    assert!(
        world.follower().printed("\x1b[31m-one"),
        "the removed line wasn't drawn red"
    );
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn draws_at_its_followers_width_for_a_program_that_never_listens_for_it() {
    let mut world = World::new();
    let file = "report-".repeat(6);

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(&format!(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && read -r; print '{file} 50%' | pbp-test downloads --once"#
    ));
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world
        .follower_width_becomes(pid.trim(), Viewer::FOLLOWER_COLUMNS)
        .await;
    running.finishes().await.succeeds();
    world
        .follower()
        .sees(&[&format!("{} 50%", &file[..34])])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_each_program_a_command_runs_in_turn() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && read -r; print 'a one' | pbp-test downloads --once; print 'b two' | pbp-test downloads --once"#,
    );
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world
        .follower_width_becomes(pid.trim(), Viewer::FOLLOWER_COLUMNS)
        .await;
    running.finishes().await.succeeds();
    world.follower().sees(&["a one", "b two"]).await;
    world.close().await;
}

#[tokio::test]
async fn reflows_a_programs_text_to_its_followers_width() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && { print 'a queued behind three other downloads today'; read -r } | pbp-test downloads"#,
    );
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world
        .follower()
        .sees(&["a queued behind three other downloads", "today"])
        .await;
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn recuts_a_programs_rows_when_its_follower_opens() {
    let mut world = World::new();
    let row = "0123456789".repeat(10);

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(&format!(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && {{ print 'a {row}'; read -r }} | pbp-test downloads"#
    ));
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world
        .follower()
        .sees(&[" 0123456789012345678901234567890123456"])
        .await;
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn recuts_a_programs_rows_when_its_follower_resizes() {
    let mut world = World::new();
    let row = "0123456789".repeat(10);

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(&format!(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && {{ print 'a {row}'; read -r }} | pbp-test downloads"#
    ));
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world
        .follower()
        .sees(&[" 0123456789012345678901234567890123456"])
        .await;
    world.follower().resize(50);
    world
        .follower()
        .sees(&[" 0123456789012345678901234567890123456789012345678"])
        .await;
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn shows_a_programs_rows_to_its_follower() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && print 'a one' | pbp-test downloads"#)
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world.follower().sees(&["a one"]).await;
    world.close().await;
}

#[tokio::test]
async fn redraws_a_programs_rows_in_place_for_its_follower() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && printf 'a one\nb two\n' | pbp-test downloads"#)
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world.follower().sees(&["a one", "b two"]).await;
    world.close().await;
}

#[tokio::test]
async fn draws_a_programs_rows_in_the_order_it_gives_them() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && printf 'a one\nc three\nb two\n' | pbp-test downloads"#)
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world.follower().sees(&["a one", "b two", "c three"]).await;
    world.close().await;
}

#[tokio::test]
async fn clears_a_row_its_program_drops_from_its_followers_view() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && printf 'a one\nb two\na\n' | pbp-test downloads"#)
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world.follower().sees(&["b two"]).await;
    world.close().await;
}

#[tokio::test]
async fn shortens_a_downloads_name_so_its_progress_fits_the_viewers_width() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer().resize(8);
    world.viewer_width_becomes(8).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && print 'report.pdf 50%' | pbp-test downloads"#)
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world.follower().sees(&["repo 50%"]).await;
    world.close().await;
}

#[tokio::test]
async fn cuts_a_programs_rows_to_the_viewers_width() {
    let mut world = World::new();
    let row = "0123456789".repeat(10);

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(&format!(
            r#"print -r -- "$PPID" && print 'a {row}' | pbp-test downloads"#
        ))
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world
        .follower()
        .sees(&[" 0123456789012345678901234567890123456789012345678901234567890123456789012345"])
        .await;
    world.close().await;
}

#[tokio::test]
async fn cuts_wide_characters_by_the_columns_they_take() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer().resize(7);
    world.viewer_width_becomes(7).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && print 'a 表表表表表' | pbp-test downloads"#)
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world.follower().sees(&[" 表表表"]).await;
    world.close().await;
}

#[tokio::test]
async fn cuts_colored_rows_by_the_columns_they_show() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer().resize(7);
    world.viewer_width_becomes(7).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && print 'a \e[32m0123456789\e[0m' | pbp-test downloads"#)
        .await;
    let pid = ran
        .succeeds()
        .printed()
        .lines()
        .next()
        .expect("the command printed run's pid");

    world.follow(pid);
    world.follower().sees(&[" 012345"]).await;
    world.close().await;
}

#[tokio::test]
async fn animates_spinners_together_while_their_program_waits() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && { printf 'a ?\nb ?\n'; read -r } | pbp-test downloads"#,
    );
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world.follower().sees(&["a ⠋", "b ⠋"]).await;
    world.follower().sees(&["a ⠙", "b ⠙"]).await;
    world.follower().sees(&["a ⠹", "b ⠹"]).await;
    world.follower().sees(&["a ⠸", "b ⠸"]).await;
    world.follower().sees(&["a ⠼", "b ⠼"]).await;
    world.follower().sees(&["a ⠴", "b ⠴"]).await;
    world.follower().sees(&["a ⠦", "b ⠦"]).await;
    world.follower().sees(&["a ⠧", "b ⠧"]).await;
    world.follower().sees(&["a ⠇", "b ⠇"]).await;
    world.follower().sees(&["a ⠏", "b ⠏"]).await;
    world.follower().sees(&["a ⠋", "b ⠋"]).await;
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn keeps_spinners_turning_when_their_program_redraws() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let mut running = world
        .claude()
        .starts(r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && pbp-test downloads"#);
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    running.types("a ?\n").await;
    world.follower().sees(&["a ⠹"]).await;
    running.types("b ?\n").await;
    world.follower().sees(&["a ⠋", "b ⠋"]).await;
    running.types("a\nb\n").await;
    world.follower().sees(&[]).await;
    world.follower().turned_spinners_in_order(&["a", "b"]);
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn fits_a_programs_rows_to_its_followers_height() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let mut running = world
        .claude()
        .starts(r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && pbp-test downloads"#);
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world.follower().resize_to(Viewer::FOLLOWER_COLUMNS, 3);
    running.types("a 1%\nb 2%\nc 3%\n").await;
    world.follower().sees(&["a 1%", "b 2%"]).await;
    running.finishes().await.succeeds();
    world.close().await;
}

#[tokio::test]
async fn debugs_the_size_a_program_is_told() {
    let mut world = World::new();

    world.installs("pbp-test");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world
        .claude()
        .starts(r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && pbp-test debug"#);
    let pid = world.workspace_file("pid").await;

    world.follow(pid.trim());
    world.follower().resize_to(40, 10);
    world.follower().sees(&["columns 40", "rows 9"]).await;
    running.finishes().await.succeeds();
    world.close().await;
}
