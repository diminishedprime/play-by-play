//! Opens a [`Broadcast`] to a viewer that hangs up before sending its width.

use std::env;
use std::io::ErrorKind;

use play_by_play::{AppError, Broadcast};
use tokio::net::UnixListener;

#[tokio::test]
async fn fails_to_open_when_the_viewer_hangs_up_before_its_width() {
    let dir = tempfile::tempdir().expect("a folder for the socket is made");
    let socket = dir.path().join("viewer.socket");
    let listener = UnixListener::bind(&socket).expect("the viewer listens");
    unsafe { env::set_var(play_by_play::env::SOCKET, &socket) };
    let viewer = tokio::spawn(async move {
        listener.accept().await.expect("the program connects");
    });

    let opened = Broadcast::<String>::open().await;
    viewer.await.expect("the viewer hangs up");

    let Err(AppError::Io(error)) = opened else {
        panic!("the broadcast opened without hearing a width");
    };
    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
}
