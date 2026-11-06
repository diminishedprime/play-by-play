//! Pretends to be a program showing its downloads live, so the e2e tests can
//! drive the play-by-play library like a real one. Its network is stdin: each
//! `file progress` line is a download's progress, and a line of just `file`
//! finishes it.

mod downloader;
mod downloads;
mod network;
mod play_by_play;

use ::play_by_play::LiveView;

use crate::downloader::Downloader;
use crate::network::Network;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let view = LiveView::open().await.expect("the live view opens");
    Downloader::new(view, Network::from_stdin()).run().await;
}
