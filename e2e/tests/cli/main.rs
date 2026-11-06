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
    world.close().await;
}

#[tokio::test]
async fn formats_a_fetched_pages_markdown() {
    let mut world = World::new();

    world
        .claude()
        .fetches(
            "https://example.com",
            "what does it say",
            "It says to take **care**.",
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
    assert_eq!(world.viewer().style_of("care"), Some(Style::Bold));
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
async fn tells_a_program_that_connects_the_viewers_width() {
    let world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;

    world
        .claude()
        .runs(r#"zmodload zsh/net/socket zsh/system && zsocket "$PLAY_BY_PLAY_SOCKET" && sysread -t 1 -c 2 -i "$REPLY" columns && print -rn -- "$columns" | xxd -p"#).await
        .succeeds()
        .prints("4d00\n");
    world.close().await;
}

#[tokio::test]
async fn finishes_a_command_whose_program_leaves_before_hearing_the_width() {
    let world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;

    world
        .claude()
        .runs(r#"zmodload zsh/net/socket && zsocket "$PLAY_BY_PLAY_SOCKET" && exec {REPLY}>&- && print done"#)
        .await
        .succeeds()
        .prints("done\n");
    world.close().await;
}

#[tokio::test]
async fn keeps_a_live_programs_lines_out_of_the_viewer() {
    let mut world = World::new();
    let claims = r#"zmodload zsh/net/socket zsh/system && zsocket "$PLAY_BY_PLAY_SOCKET" && sysread -t 1 -c 2 -i "$REPLY" && print live"#;

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    world
        .claude()
        .runs(claims)
        .await
        .succeeds()
        .prints("live\n");

    world
        .viewer()
        .sees(&[&format!("$ {claims}"), "exit: 0"])
        .await;
    world.close().await;
}

#[tokio::test]
async fn shows_a_live_programs_view_to_its_follower() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"zmodload zsh/net/socket zsh/system && zsocket "$PLAY_BY_PLAY_SOCKET" && fd=$REPLY && sysread -t 1 -c 2 -i "$fd" columns && print -rn -- hello >&"$fd" && print -r -- "$PPID""#).await;
    let pid = ran.succeeds().printed().trim();

    world.follow(pid);
    world.follower().sees(&["hello"]).await;
    world.close().await;
}

#[tokio::test]
async fn keeps_showing_a_live_programs_view_as_it_changes() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world
        .claude()
        .starts(r#"zmodload zsh/net/socket zsh/system && zsocket "$PLAY_BY_PLAY_SOCKET" && fd=$REPLY && sysread -t 1 -c 2 -i "$fd" columns && print -rn -- first >&"$fd" && print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && read -r; print -rn -- second >&"$fd""#);
    let pid = world.workspace_file("pid").await;
    world.follow(pid.trim());

    world.follower().sees(&["first"]).await;
    running.finishes().await.succeeds();
    world.follower().sees(&["firstsecond"]).await;
    world.close().await;
}

#[tokio::test]
async fn tells_a_live_program_its_followers_width() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world
        .claude()
        .starts(r#"zmodload zsh/net/socket zsh/system && zsocket "$PLAY_BY_PLAY_SOCKET" && fd=$REPLY && sysread -t 1 -c 2 -i "$fd" first && print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && sysread -t 2 -c 2 -i "$fd" second && print -rn -- "$first$second" | xxd -p"#);
    let pid = world.workspace_file("pid").await;
    world.follow(pid.trim());

    running.finishes().await.succeeds().prints("4d002600\n");
    world.close().await;
}

#[tokio::test]
async fn tells_a_live_program_when_its_follower_resizes() {
    let mut world = World::new();

    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world
        .claude()
        .starts(r#"zmodload zsh/net/socket zsh/system && zsocket "$PLAY_BY_PLAY_SOCKET" && fd=$REPLY && sysread -t 1 -c 2 -i "$fd" first && print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && sysread -t 2 -c 2 -i "$fd" second && print > followed.tmp && mv followed.tmp followed && sysread -t 2 -c 2 -i "$fd" third && print -rn -- "$first$second$third" | xxd -p"#);
    let pid = world.workspace_file("pid").await;
    world.follow(pid.trim());
    world.workspace_file("followed").await;
    world.follower().resize(50);

    running.finishes().await.succeeds().prints("4d0026003200\n");
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
async fn prints_a_programs_rows_as_plain_lines_while_nobody_watches() {
    let world = World::unwatched();

    world.installs("live-view");

    world
        .claude()
        .runs(r"printf 'a one\nb two\n' | live-view")
        .await
        .succeeds()
        .prints("downloading a\ndownloading b\n");
    world.close().await;
}

#[tokio::test]
async fn prints_a_programs_row_again_only_when_it_changes() {
    let world = World::unwatched();

    world.installs("live-view");

    world
        .claude()
        .runs(r"printf 'a one\na uno\nb two\n' | live-view")
        .await
        .succeeds()
        .prints("downloading a\ndownloading b\n");
    world.close().await;
}

#[tokio::test]
async fn narrates_each_download_once_however_often_it_progresses() {
    let world = World::unwatched();

    world.installs("live-view");

    world
        .claude()
        .runs("print -l 'a 1%' 'a 10%' 'a 95%' 'a 100%' 'a' | live-view")
        .await
        .succeeds()
        .prints("downloading a\ndownloaded a\n");
    world.close().await;
}

#[tokio::test]
async fn follows_a_program_that_claims_the_viewer_after_its_follower_opens() {
    let mut world = World::new();

    world.installs("live-view");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && read -r; print 'a one' | live-view"#,
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
async fn recuts_a_programs_rows_when_its_follower_opens() {
    let mut world = World::new();
    let row = "0123456789".repeat(10);

    world.installs("live-view");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let running = world.claude().starts(&format!(
        r#"print -r -- "$PPID" > pid.tmp && mv pid.tmp pid && {{ print 'a {row}'; read -r }} | live-view"#
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
async fn keeps_printing_a_programs_rows_once_its_terminal_goes() {
    let world = World::unwatched();

    world.installs("live-view");

    world
        .claude()
        .runs(
            r#"zmodload zsh/net/socket zsh/system && zsocket -l fake.socket && listener=$REPLY && {
    zsocket -a "$listener" && fd=$REPLY
    printf '\x4d\x00' >&"$fd"
    print 'a one'
    sysread -t 1 -i "$fd" frame
    exec {fd}>&-
    print 'b two'
} | PLAY_BY_PLAY_SOCKET=fake.socket live-view"#,
        )
        .await
        .succeeds()
        .prints("downloading a\ndownloading b\n");
    world.close().await;
}

#[tokio::test]
async fn shows_a_programs_rows_to_its_follower() {
    let mut world = World::new();

    world.installs("live-view");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && print 'a one' | live-view"#)
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

    world.installs("live-view");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && printf 'a one\nb two\n' | live-view"#)
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
async fn clears_a_row_its_program_drops_from_its_followers_view() {
    let mut world = World::new();

    world.installs("live-view");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && printf 'a one\nb two\na\n' | live-view"#)
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

    world.installs("live-view");
    world.viewer().resize(8);
    world.viewer_width_becomes(8).await;
    let ran = world
        .claude()
        .runs(r#"print -r -- "$PPID" && print 'report.pdf 50%' | live-view"#)
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

    world.installs("live-view");
    world.viewer_width_becomes(Viewer::COLUMNS).await;
    let ran = world
        .claude()
        .runs(&format!(
            r#"print -r -- "$PPID" && print 'a {row}' | live-view"#
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
