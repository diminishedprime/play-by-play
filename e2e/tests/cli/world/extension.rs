use std::path::Path;
use std::time::Duration;

use play_by_play::proto::Claimed;
use prost::Message as _;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::task::JoinHandle;
use tokio::time;

pub struct Extension {
    heard: JoinHandle<Claimed>,
}

impl Extension {
    pub fn listen(socket: &Path) -> Extension {
        let listener = UnixListener::bind(socket).expect("the fake extension listens");
        let heard = tokio::spawn(async move {
            let (mut pbp, _) = listener.accept().await.expect("pbp connects");
            let length = pbp.read_u32().await.expect("pbp sends a frame's length");
            let mut message =
                vec![0; usize::try_from(length).expect("a frame's length fits in memory")];
            pbp.read_exact(&mut message)
                .await
                .expect("pbp sends the whole frame");
            Claimed::decode(message.as_slice()).expect("pbp sends a Claimed")
        });
        Extension { heard }
    }

    pub async fn heard_claimed(self, pid: &str, command: &str) {
        let heard = time::timeout(Duration::from_secs(1), self.heard)
            .await
            .expect("pbp tells the extension within a second")
            .expect("the fake extension hears pbp");
        assert_eq!(
            (heard.pid.to_string(), heard.command),
            (pid.to_owned(), command.to_owned())
        );
    }
}
