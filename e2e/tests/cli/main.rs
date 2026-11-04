//! Runs the `play-by-play` binary named by `PLAY_BY_PLAY_BIN`, which
//! `just e2e test` builds and sets.

mod doctor;
mod world;

use std::path::Path;

use world::{Style, Viewer, World};

#[tokio::test]
async fn reports_its_version_for_the_plugins_install_check() {
    let world = World::new();

    world
        .sut(["--version"])
        .run()
        .succeeds()
        .prints("play-by-play 0.1.0\n");
}

#[tokio::test]
async fn numbers_a_partial_read_from_its_first_line() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world
        .claude()
        .reads_from("/notes.md", "forty\nforty-one\n", 40);

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
}

#[tokio::test]
async fn shows_the_viewer_the_file_claude_wrote() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer_theme_becomes("Solarized (dark)").await;
    world
        .claude()
        .writes("/main.rs", "fn main() {\n    let x = 1;\n}\n");

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
}

#[tokio::test]
async fn colors_an_edits_diff() {
    let mut world = World::new();

    world.viewer_theme_becomes("Solarized (dark)").await;
    world.claude().edits(
        "/main.rs",
        &[" fn main() {", "-    let x = 1;", "+    let x = 2;", " }"],
    );

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
}

#[tokio::test]
async fn unwraps_a_wrapped_command_for_cc_toolgate() {
    let world = World::new();

    world
        .sut(["unwrap"])
        .stdin("pbp -c 'echo '\\''hi'\\'''")
        .run()
        .succeeds()
        .prints("echo 'hi'");
}

#[tokio::test]
async fn leaves_a_command_it_didnt_wrap_as_it_is() {
    let world = World::new();

    world
        .sut(["unwrap"])
        .stdin("ls")
        .run()
        .succeeds()
        .prints("ls");
}

#[tokio::test]
async fn watches_in_the_workspaces_tmux_session() {
    let world = World::new();

    world
        .sut(["watch", "--tmux"])
        .project(Path::new("/Users/me/programming/notes"))
        .run()
        .succeeds();

    world.tmux_ran(&[
        "new-session",
        "-A",
        "-s",
        "notes_75326a409771",
        "tmux set-option destroy-unattached off && play-by-play watch",
    ]);
}

#[tokio::test]
async fn shows_the_viewer_the_file_claude_read() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer_theme_becomes("Solarized (dark)").await;
    world
        .claude()
        .reads("/main.rs", "fn main() {\n    let x = 1;\n}\n");

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
}

#[tokio::test]
async fn shows_the_viewer_a_command_claude_ran() {
    let mut world = World::new();

    world
        .claude()
        .runs("echo out; exit 3")
        .exits(3)
        .prints("out\n")
        .prints_error("");

    world
        .viewer()
        .sees(&["$ echo out; exit 3", "out", "exit: 3"])
        .await;
}

#[tokio::test]
async fn formats_an_agents_markdown() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer_theme_becomes("Solarized (dark)").await;
    world.claude().asks_agent(
        "find **everything**",
        r"## Overview

The viewer follows the log with `kqueue`, so take **care**: every write reaches it straight away and nothing waits for the command to finish.

- first item
- a second item that is long enough to wrap past the viewer's seventy-seven columns and keep going on its way

```rust
let color = log.next_color()?;
```

```
plain code
```
",
    );

    world
        .viewer()
        .sees(&[
            "agent: explore",
            "prompt:",
            "find everything",
            "response:",
            "Overview",
            "",
            "The viewer follows the log with kqueue, so take care: every write reaches it",
            "straight away and nothing waits for the command to finish.",
            "",
            "- first item",
            "- a second item that is long enough to wrap past the viewer's seventy-seven",
            "  columns and keep going on its way",
            "",
            "let color = log.next_color()?;",
            "",
            "plain code",
            "agent: end",
        ])
        .await;
    assert_eq!(world.viewer().style_of("everything"), Some(Style::Bold));
    assert_eq!(world.viewer().style_of("- "), Some(Style::Cyan));
    assert_eq!(world.viewer().style_of("first"), Some(Style::Cyan));
    assert_eq!(world.viewer().style_of("Overview"), Some(Style::Blue));
    assert_eq!(world.viewer().style_of("kqueue"), Some(Style::Green));
    assert_eq!(world.viewer().style_of("care"), Some(Style::Bold));
    assert_eq!(world.viewer().style_of("let"), Some(Style::SolarizedBlue));
}

#[tokio::test]
async fn formats_a_fetched_pages_markdown() {
    let mut world = World::new();

    world.claude().fetches(
        "https://example.com",
        "what does it say",
        "It says to take **care**.",
    );

    world
        .viewer()
        .sees(&[
            "fetch: https://example.com",
            "prompt: what does it say",
            "It says to take care.",
            "fetch: end",
        ])
        .await;
    assert_eq!(world.viewer().style_of("care"), Some(Style::Bold));
}

#[tokio::test]
async fn formats_a_searchs_summary_markdown() {
    let mut world = World::new();

    world.claude().searches(
        "file watching",
        &[("notify crate docs", "https://docs.rs/notify")],
        "Use **kqueue** to watch files.",
    );

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
}

#[tokio::test]
async fn picks_the_code_theme_closest_to_the_viewers_background() {
    let world = World::new();

    world.viewer_theme_becomes("Solarized (dark)").await;
}

#[tokio::test]
async fn records_the_viewers_width() {
    let world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
}

#[tokio::test]
async fn records_the_viewers_width_when_it_resizes() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.viewer().resize(100);

    world.viewer_width_becomes(100).await;
}

#[tokio::test]
async fn gives_each_tool_call_the_next_color() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world.claude().reads("/notes.md", "first\n");
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
    world.claude().runs("echo hi").succeeds();
    world.viewer().sees(&["$ echo hi", "hi", "exit: 0"]).await;

    assert_eq!(world.viewer().colors(), [Style::Green, Style::Yellow]);
}

#[tokio::test]
async fn shows_a_failed_commands_exit_in_red() {
    let mut world = World::new();

    world.claude().runs("exit 3").exits(3);
    world.viewer().sees(&["$ exit 3", "exit: 3"]).await;

    assert_eq!(world.viewer().exit_colors(), [Style::Red]);
}

#[tokio::test]
async fn shows_a_successful_commands_exit_in_its_headers_color() {
    let mut world = World::new();

    world.claude().runs("true").succeeds();
    world.viewer().sees(&["$ true", "exit: 0"]).await;

    assert_eq!(world.viewer().exit_colors(), [Style::Green]);
}

#[tokio::test]
async fn shows_the_viewer_a_command_while_it_runs() {
    let mut world = World::new();

    let running = world.claude().starts("echo started; read; echo finished");

    world
        .viewer()
        .sees(&["$ echo started; read; echo finished", "started"])
        .await;
    running.finishes().succeeds();
}

#[tokio::test]
async fn shows_the_viewer_a_commands_errors_and_keeps_them_on_stderr() {
    let mut world = World::new();

    world
        .claude()
        .runs("echo err >&2")
        .exits(0)
        .prints("")
        .prints_error("err\n");

    world
        .viewer()
        .sees(&["$ echo err >&2", "err", "exit: 0"])
        .await;
}
