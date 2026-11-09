//! Pretends to be a program using the play-by-play library, so the e2e tests
//! can drive it like a real one. Each subcommand is a different program.

mod debug;
mod downloader;
mod downloads;
mod network;
mod play_by_play;

use ::play_by_play::Broadcast;
use clap::{Parser, Subcommand};

use crate::debug::Debug;
use crate::downloader::Downloader;
use crate::network::Network;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    program: Program,
}

#[derive(Subcommand)]
enum Program {
    /// Shows its downloads live. Its network is stdin: each `file progress`
    /// line is a download's progress, and a line of just `file` finishes it.
    /// A progress of `old->new` shows the diff between the two.
    Downloads {
        /// Reads all of stdin before rendering a single time, never listening
        /// for the viewer's width.
        #[arg(long)]
        once: bool,
    },
    /// Shows what the library tells it about the viewer, until stdin closes.
    Debug,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    match Cli::parse().program {
        Program::Downloads { once } => {
            let broadcast = Broadcast::open().await.expect("the broadcast opens");
            let downloader = Downloader::new(broadcast, Network::from_stdin());
            if once {
                downloader.run_once().await;
            } else {
                downloader.run().await;
            }
        }
        Program::Debug => {
            Debug::run(Broadcast::open().await.expect("the broadcast opens")).await;
        }
    }
}
