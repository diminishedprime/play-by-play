use ::play_by_play::LiveView;

use crate::downloads::Downloads;
use crate::network::{Event, Network};

pub struct Downloader {
    view: LiveView<String>,
    downloads: Downloads,
    network: Network,
}

impl Downloader {
    pub fn new(view: LiveView<String>, network: Network) -> Downloader {
        Downloader {
            view,
            downloads: Downloads::default(),
            network,
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.network.next() => match event {
                    Some(Event::Progress { file, progress }) => self.downloads.progress(file, progress),
                    Some(Event::Finished { file }) => self.downloads.finish(&file),
                    None => return,
                },
                resized = self.view.resized() => resized.expect("the viewer's width arrives"),
            }
            self.view
                .render(&self.downloads)
                .await
                .expect("the live view renders");
        }
    }
}
