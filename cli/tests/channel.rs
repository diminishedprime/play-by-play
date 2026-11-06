//! Drives a [`Channel`] over a socket the test plays the program's end of.

use std::fs;
use std::io::Read;
use std::os::unix::net::UnixStream;

use play_by_play_cli::channel::Channel;

#[tokio::test]
async fn sends_the_viewers_width_to_a_program_that_already_left() {
    let dir = tempfile::tempdir().expect("a folder for the socket is made");
    let mut channel = Channel::open(dir.path()).expect("the channel opens");
    let program = UnixStream::connect(channel.path()).expect("the program connects");
    channel.accept().await.expect("the program is accepted");
    drop(program);

    channel
        .send_columns(77)
        .await
        .expect("the width goes nowhere without failing");
}

#[tokio::test]
async fn finishes_serving_a_program_that_left_before_its_followers_width() {
    let dir = tempfile::tempdir().expect("a folder for the socket is made");
    let follower = dir.path().join("follower");
    fs::create_dir(&follower).expect("the follower's folder is made");
    let columns = follower.join("columns");
    fs::write(&columns, "50").expect("the follower's width is recorded");
    let mut channel = Channel::open(dir.path()).expect("the channel opens");
    let mut program = UnixStream::connect(channel.path()).expect("the program connects");
    channel.accept().await.expect("the program is accepted");
    channel
        .send_columns(77)
        .await
        .expect("the viewer's width is sent");
    let mut width = [0; 6];
    program
        .read_exact(&mut width)
        .expect("the program hears the viewer's width");
    drop(program);

    channel
        .serve(Vec::new(), &follower, &columns)
        .await
        .expect("serving finishes though the follower's width can't be sent");
}
