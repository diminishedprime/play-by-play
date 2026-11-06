//! Renders a [`Broadcast`] to a viewer the test plays over a socket.

use std::env;
use std::fmt;

use play_by_play::{Board, Broadcast, Live};
use tokio::io::AsyncWriteExt;
use tokio::net::UnixListener;

struct Row(String);

impl Live for Row {
    type Key = String;

    fn key(&self) -> &String {
        &self.0
    }

    fn show(&self, _columns: u16, out: &mut impl fmt::Write) -> fmt::Result {
        out.write_str(&self.0)
    }
}

struct Rows(Vec<Row>);

impl Board for Rows {
    type Key = String;
    type Line<'a> = &'a Row;

    fn lines(&self) -> impl Iterator<Item = &Row> {
        self.0.iter()
    }
}

#[tokio::test]
async fn keeps_rendering_once_its_viewer_goes() {
    let dir = tempfile::tempdir().expect("a folder for the socket is made");
    let socket = dir.path().join("viewer.socket");
    let listener = UnixListener::bind(&socket).expect("the viewer listens");
    unsafe { env::set_var(play_by_play::env::SOCKET, &socket) };
    let viewer = tokio::spawn(async move {
        let (mut program, _) = listener.accept().await.expect("the program connects");
        program
            .write_all(b"\x00\x00\x00\x02\x08\x4d")
            .await
            .expect("the viewer sends its width");
    });

    let mut broadcast = Broadcast::<String>::open()
        .await
        .expect("the broadcast opens");
    viewer.await.expect("the viewer hangs up");
    let rows = Rows(vec![Row("a one".to_owned())]);

    tokio::select! {
        biased;
        resized = broadcast.resized() => panic!("a gone viewer resized: {resized:?}"),
        () = async {} => {}
    }
    broadcast
        .render(&rows)
        .await
        .expect("the broadcast renders as its viewer goes");
    broadcast
        .render(&rows)
        .await
        .expect("the broadcast keeps rendering without a viewer");
}
