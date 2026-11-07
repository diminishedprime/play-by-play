//! `play-by-play` under a shorter name, for the `pbp -c` the plugin wraps
//! Claude's commands in.

use std::process::ExitCode;

use play_by_play_cli::cmd::Cli;
use play_by_play_cli::error::AppExit;

fn main() -> AppExit<ExitCode> {
    play_by_play_cli::main(Cli::parse_args()).into()
}
