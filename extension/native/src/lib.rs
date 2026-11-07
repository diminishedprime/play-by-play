//! The VS Code extension's native side.

use std::fs;
use std::path::PathBuf;

use napi::Status;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use play_by_play::AppResult;
use play_by_play::log::Log;
use play_by_play::proto;
use prost::Message as _;
use tokio::io::AsyncReadExt;
use tokio::net::UnixListener;
use tokio::task::JoinHandle;

type OnClaimed = ThreadsafeFunction<Claimed, (), Claimed, Status, false, true>;

/// A command whose program claimed the viewer.
#[napi(object)]
pub struct Claimed {
    /// The pid of the play-by-play the program is live under.
    pub pid: u32,
    /// The command Claude ran.
    pub command: String,
}

/// Hearing which commands claim the viewer, until disposed.
#[napi]
pub struct Claims {
    hearing: JoinHandle<AppResult>,
    socket: PathBuf,
}

#[napi]
impl Claims {
    /// Stops hearing claims.
    #[napi]
    pub fn dispose(&self) {
        self.hearing.abort();
        let _ = fs::remove_file(&self.socket);
    }

    async fn hear(listener: UnixListener, on_claimed: OnClaimed) -> AppResult {
        loop {
            let (mut pbp, _) = listener.accept().await?;
            let length = pbp.read_u32().await?;
            let mut message = vec![0; usize::try_from(length)?];
            pbp.read_exact(&mut message).await?;
            let claimed = proto::Claimed::decode(message.as_slice())?;
            on_claimed.call(
                Claimed {
                    pid: claimed.pid,
                    command: claimed.command,
                },
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        }
    }
}

/// Hears which commands claim the viewer of the workspace at `root`, handing
/// each to `on_claimed`.
///
/// # Errors
///
/// When the workspace's socket can't be made.
#[napi(async_runtime)]
pub fn listen(
    root: String,
    on_claimed: ThreadsafeFunction<Claimed, (), Claimed, Status, false, true>,
) -> napi::Result<Claims> {
    let folder = Log::folder();
    let root = PathBuf::from(root);
    let socket = Log::for_workspace(&folder, &root).extension_socket_path();
    fs::create_dir_all(&folder)?;
    let _ = fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket)?;
    let hearing = tokio::spawn(Claims::hear(listener, on_claimed));
    Ok(Claims { hearing, socket })
}
