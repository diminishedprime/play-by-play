//! Pretends to be a program showing its downloads live, so the e2e tests can
//! drive the play-by-play library like a real one. Its network is stdin: each
//! `file progress` line is a download's progress, and a line of just `file`
//! finishes it. With `--once` it reads all of stdin before rendering a single
//! time, never listening for the viewer's width.

mod downloader;
mod downloads;
mod network;
mod play_by_play;

use ::play_by_play::Broadcast;

use crate::downloader::Downloader;
use crate::network::Network;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let broadcast = Broadcast::open().await.expect("the broadcast opens");
    let downloader = Downloader::new(broadcast, Network::from_stdin());
    if std::env::args().any(|arg| arg == "--once") {
        downloader.run_once().await;
    } else {
        downloader.run().await;
    }
}
