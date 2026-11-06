//! Compiles `broadcast.proto` into the library's `proto` module.

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=broadcast.proto");
    let files = protox::compile(["broadcast.proto"], ["."])?;
    prost_build::Config::new().compile_fds(files)?;
    Ok(())
}
