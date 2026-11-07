//! The socket the VS Code extension listens on, to hear which commands claim
//! the viewer.

use std::path::Path;
use std::process;

use bytes::{Bytes, BytesMut};
use play_by_play::proto;
use prost::Message as _;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio_util::codec::{Encoder as _, LengthDelimitedCodec};

use crate::error::AppResult;

/// The VS Code extension, as play-by-play tells it about claims.
pub struct Extension;

impl Extension {
    /// Tells the extension listening at `socket` that the program running
    /// `command` under this play-by-play claimed the viewer, unless nothing
    /// listens there.
    ///
    /// # Errors
    ///
    /// When the extension can't be written to.
    pub async fn tell_claimed(socket: &Path, command: &str) -> AppResult {
        let Ok(mut extension) = UnixStream::connect(socket).await else {
            return Ok(());
        };
        let claimed = proto::Claimed {
            pid: process::id(),
            command: command.to_owned(),
        };
        let mut message = BytesMut::new();
        LengthDelimitedCodec::new().encode(Bytes::from(claimed.encode_to_vec()), &mut message)?;
        extension.write_all(&message).await?;
        Ok(())
    }
}
