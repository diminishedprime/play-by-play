//! Runs Claude's commands so a viewer can follow them live, and is that viewer.

use play_by_play_cli::cmd::Cli;
use play_by_play_cli::error::AppExit;

fn main() -> AppExit {
    play_by_play_cli::main(Cli::parse_args()).into()
}
