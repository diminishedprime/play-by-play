use tokio::io::{self, AsyncBufReadExt, BufReader, Lines, Stdin};

pub enum Event {
    Progress { file: String, progress: String },
    Finished { file: String },
}

pub struct Network {
    input: Lines<BufReader<Stdin>>,
}

impl Network {
    pub fn from_stdin() -> Network {
        Network {
            input: BufReader::new(io::stdin()).lines(),
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        let line = self.input.next_line().await.expect("stdin reads")?;
        Some(match line.split_once(' ') {
            Some((file, progress)) => Event::Progress {
                file: file.to_owned(),
                progress: progress.to_owned(),
            },
            None => Event::Finished { file: line },
        })
    }
}
