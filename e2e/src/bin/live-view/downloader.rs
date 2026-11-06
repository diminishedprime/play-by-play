use ::play_by_play::Broadcast;

use crate::downloads::Downloads;
use crate::network::{Event, Network};

pub struct Downloader {
    broadcast: Broadcast<String>,
    downloads: Downloads,
    network: Network,
}

impl Downloader {
    pub fn new(broadcast: Broadcast<String>, network: Network) -> Downloader {
        Downloader {
            broadcast,
            downloads: Downloads::default(),
            network,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.network.next() => match event {
                    Some(Event::Progress { file, progress }) => self.progress(file, progress),
                    Some(Event::Finished { file }) => self.downloads.finish(&file),
                    None => return,
                },
                resized = self.broadcast.resized() => resized.expect("the viewer's width arrives"),
            }
            self.broadcast
                .render(&self.downloads)
                .await
                .expect("the broadcast renders");
        }
    }

    fn progress(&mut self, file: String, progress: String) {
        if !self.downloads.has(&file) {
            println!("downloading {file}");
        }
        if progress == "100%" {
            println!("downloaded {file}");
        }
        self.downloads.progress(file, progress);
    }
}
