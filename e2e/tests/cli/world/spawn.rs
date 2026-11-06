use std::env;
use std::ffi::{CString, OsStr, c_char, c_int, c_short};
use std::io;
use std::iter;
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, OwnedFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::ExitStatusExt;
use std::process::{Command, ExitStatus, Output};
use std::ptr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::pipe::{Receiver, Sender};
use tokio::signal::unix::{self, Signal, SignalKind};

pub enum Stdio {
    Pipe,
    Fd(OwnedFd),
    Null,
}

pub struct Spawned {
    pid: Option<libc::pid_t>,
    exits: Signal,
    stdin: Option<Sender>,
    stdout: Option<Receiver>,
    stderr: Option<Receiver>,
}

impl Spawned {
    pub fn new(command: &Command, [stdin, stdout, stderr]: [Stdio; 3]) -> Spawned {
        let exits = unix::signal(SignalKind::child()).expect("child exits can be awaited");
        let program = Spawned::c_string(command.get_program());
        let arguments = iter::once(command.get_program())
            .chain(command.get_args())
            .map(Spawned::c_string)
            .collect::<Vec<_>>();
        let environment = Spawned::environment(command);
        let cwd = command
            .get_current_dir()
            .map(|cwd| Spawned::c_string(cwd.as_os_str()));
        let mut actions: libc::posix_spawn_file_actions_t = ptr::null_mut();
        let mut attrs: libc::posix_spawnattr_t = ptr::null_mut();
        let mut pid = 0;
        let (stdin, stdout, stderr) = unsafe {
            Spawned::check(
                libc::posix_spawn_file_actions_init(&raw mut actions),
                "spawn actions start",
            );
            let stdin = Spawned::slot(&mut actions, 0, stdin);
            let stdout = Spawned::slot(&mut actions, 1, stdout);
            let stderr = Spawned::slot(&mut actions, 2, stderr);
            if let Some(cwd) = &cwd {
                Spawned::check(
                    libc::posix_spawn_file_actions_addchdir_np(&raw mut actions, cwd.as_ptr()),
                    "the child's folder is set",
                );
            }
            Spawned::check(
                libc::posix_spawnattr_init(&raw mut attrs),
                "spawn attributes start",
            );
            let mut defaults = MaybeUninit::<libc::sigset_t>::uninit();
            libc::sigemptyset(defaults.as_mut_ptr());
            libc::sigaddset(defaults.as_mut_ptr(), libc::SIGPIPE);
            Spawned::check(
                libc::posix_spawnattr_setsigdefault(&raw mut attrs, defaults.as_ptr()),
                "SIGPIPE is reset for the child",
            );
            let mut mask = MaybeUninit::<libc::sigset_t>::uninit();
            libc::sigemptyset(mask.as_mut_ptr());
            Spawned::check(
                libc::posix_spawnattr_setsigmask(&raw mut attrs, mask.as_ptr()),
                "the child's signal mask is cleared",
            );
            let flags = libc::POSIX_SPAWN_CLOEXEC_DEFAULT
                | libc::POSIX_SPAWN_SETSIGDEF
                | libc::POSIX_SPAWN_SETSIGMASK;
            Spawned::check(
                libc::posix_spawnattr_setflags(
                    &raw mut attrs,
                    c_short::try_from(flags).expect("the spawn flags fit a short"),
                ),
                "the spawn flags are set",
            );
            let argv = Spawned::pointers(&arguments);
            let envp = Spawned::pointers(&environment);
            Spawned::check(
                libc::posix_spawn(
                    &raw mut pid,
                    program.as_ptr(),
                    &raw const actions,
                    &raw const attrs,
                    argv.as_ptr(),
                    envp.as_ptr(),
                ),
                "the program spawns",
            );
            libc::posix_spawn_file_actions_destroy(&raw mut actions);
            libc::posix_spawnattr_destroy(&raw mut attrs);
            (stdin, stdout, stderr)
        };
        Spawned {
            pid: Some(pid),
            exits,
            stdin: stdin
                .0
                .map(|fd| Sender::from_owned_fd(fd).expect("the child's stdin can be awaited")),
            stdout: stdout
                .0
                .map(|fd| Receiver::from_owned_fd(fd).expect("the child's stdout can be awaited")),
            stderr: stderr
                .0
                .map(|fd| Receiver::from_owned_fd(fd).expect("the child's stderr can be awaited")),
        }
    }

    pub async fn output(mut self, input: Vec<u8>) -> Output {
        let stdin = self.stdin.take();
        let stdout = self.stdout.take();
        let stderr = self.stderr.take();
        let ((), stdout, stderr, status) = tokio::join!(
            Spawned::write_all(stdin, &input),
            Spawned::read_all(stdout),
            Spawned::read_all(stderr),
            self.exit(),
        );
        Output {
            status,
            stdout,
            stderr,
        }
    }

    pub async fn kill(&mut self) {
        let pid = self.pid.expect("the child is still running");
        unsafe { libc::kill(pid, libc::SIGKILL) };
        self.exit().await;
    }

    async fn exit(&mut self) -> ExitStatus {
        let pid = self.pid.expect("the child is still running");
        loop {
            if let Some(status) = Spawned::reaped(pid) {
                self.pid = None;
                return status;
            }
            self.exits
                .recv()
                .await
                .expect("child exits keep being reported");
        }
    }

    unsafe fn slot(
        actions: &mut libc::posix_spawn_file_actions_t,
        target: c_int,
        stdio: Stdio,
    ) -> (Option<OwnedFd>, Option<OwnedFd>) {
        match stdio {
            Stdio::Pipe => {
                let (reader, writer) = io::pipe().expect("a pipe opens");
                let (parent, child): (OwnedFd, OwnedFd) = if target == 0 {
                    (writer.into(), reader.into())
                } else {
                    (reader.into(), writer.into())
                };
                Spawned::check(
                    unsafe {
                        libc::posix_spawn_file_actions_adddup2(actions, child.as_raw_fd(), target)
                    },
                    "a pipe becomes the child's stdio",
                );
                (Some(parent), Some(child))
            }
            Stdio::Fd(fd) => {
                Spawned::check(
                    unsafe {
                        libc::posix_spawn_file_actions_adddup2(actions, fd.as_raw_fd(), target)
                    },
                    "a descriptor becomes the child's stdio",
                );
                (None, Some(fd))
            }
            Stdio::Null => {
                Spawned::check(
                    unsafe {
                        libc::posix_spawn_file_actions_addopen(
                            actions,
                            target,
                            c"/dev/null".as_ptr(),
                            libc::O_RDWR,
                            0,
                        )
                    },
                    "/dev/null becomes the child's stdio",
                );
                (None, None)
            }
        }
    }

    fn environment(command: &Command) -> Vec<CString> {
        let changes = command.get_envs().collect::<Vec<_>>();
        env::vars_os()
            .filter(|(key, _)| !changes.iter().any(|(changed, _)| changed == key))
            .map(|(key, value)| Spawned::entry(&key, &value))
            .chain(
                changes
                    .iter()
                    .filter_map(|(key, value)| value.map(|value| Spawned::entry(key, value))),
            )
            .collect()
    }

    fn entry(key: &OsStr, value: &OsStr) -> CString {
        CString::new([key.as_bytes(), b"=", value.as_bytes()].concat())
            .expect("an environment entry has no NUL")
    }

    fn c_string(text: &OsStr) -> CString {
        CString::new(text.as_bytes()).expect("a spawn argument has no NUL")
    }

    fn pointers(strings: &[CString]) -> Vec<*mut c_char> {
        strings
            .iter()
            .map(|string| string.as_ptr().cast_mut())
            .chain(iter::once(ptr::null_mut()))
            .collect()
    }

    async fn write_all(stdin: Option<Sender>, input: &[u8]) {
        if let Some(mut stdin) = stdin {
            let _ = stdin.write_all(input).await;
        }
    }

    async fn read_all(pipe: Option<Receiver>) -> Vec<u8> {
        let mut read = Vec::new();
        if let Some(mut pipe) = pipe {
            pipe.read_to_end(&mut read)
                .await
                .expect("the child's output reads");
        }
        read
    }

    fn reaped(pid: libc::pid_t) -> Option<ExitStatus> {
        let mut status = 0;
        loop {
            match unsafe { libc::waitpid(pid, &raw mut status, libc::WNOHANG) } {
                0 => return None,
                reaped if reaped == pid => return Some(ExitStatus::from_raw(status)),
                _ => {
                    let error = io::Error::last_os_error();
                    assert_eq!(
                        error.kind(),
                        io::ErrorKind::Interrupted,
                        "the child is waited on: {error}"
                    );
                }
            }
        }
    }

    fn check(code: c_int, what: &str) {
        assert_eq!(code, 0, "{what}: {}", io::Error::from_raw_os_error(code));
    }
}

impl Drop for Spawned {
    fn drop(&mut self) {
        if let Some(pid) = self.pid.take() {
            unsafe { libc::kill(pid, libc::SIGKILL) };
        }
    }
}
