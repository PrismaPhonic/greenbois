#[macro_use]
extern crate failure;

/// committer contains methods for issuing git commits.
pub mod committer;
/// custom in-house errors that we translate to from other errors received by external crates.
pub mod errors;
/// options contains Structopt enum for parsing terminal commands and providing helpful menus.
pub mod options;
/// writer contains methods for building and manipulating git blobs.
pub mod writer;
/// hasher contains methods for hashing a blob.
pub mod hasher;

use crate::committer::Committer;
use crate::errors::GitTerminalError;
pub use crate::options::{Options, Opts};
use std::process::Command;

use failure::Error;

/// Calling this function from a binary program will cause it to match on the commands
/// passed by the user, and run the appropriate internal functions.
pub fn run(config: Opts) -> Result<(), Error> {
    match config {
        Opts::Commit { repo, msg, yrs_ago } => commit(Options { repo, msg, yrs_ago }),
    }
}

fn commit(opts: Options) -> Result<(), Error> {
    let committer = Committer::new(opts)?;
    committer.commit_all()?;
    println!("Successfully committed.");
    Ok(())
}
