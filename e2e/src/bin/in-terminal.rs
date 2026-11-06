//! Runs its arguments as a command in a new session whose controlling terminal
//! is stdin's, so the e2e tests can give `watch` a terminal without `pre_exec`.

use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

use rustix::process;

fn main() {
    process::setsid().expect("a new session starts");
    process::ioctl_tiocsctty(io::stdin()).expect("stdin's terminal becomes the controlling one");
    let mut args = env::args_os().skip(1);
    let program = args.next().expect("a command to run follows");
    let error = Command::new(program).args(args).exec();
    panic!("the command can't run: {error}");
}
