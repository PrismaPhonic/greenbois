
/// committer contains methods for issuing git commits.
pub mod committer;
/// dates contains constants of holidays and logic around which days and times to skip committing on.
pub mod dates;
/// custom in-house errors that we translate to from other errors received by external crates.
pub mod errors;
/// options contains Structopt enum for parsing terminal commands and providing helpful menus.
pub mod options;
/// writer contains methods for building and manipulating git blobs.
pub mod writer;

use crate::committer::Committer;
pub use crate::options::{Options, Opts};

use chrono::NaiveTime;
use crate::errors::Error;

/// Calling this function from a binary program will cause it to match on the commands
/// passed by the user, and run the appropriate internal functions.
pub fn run(config: Opts) -> Result<(), Error> {
    match config {
        Opts::Commit {
            repo,
            msg,
            yrs_ago,
            start,
            end,
        } => commit(Options {
            repo,
            msg,
            yrs_ago,
            // TODO: Deal with invalid times.
            start: NaiveTime::from_hms_opt(start, 0, 0).expect("Invalid time"),
            end: NaiveTime::from_hms_opt(end, 0, 0).expect("Invalid time"),
        }),
    }
}

fn commit(opts: Options) -> Result<(), Error> {
    let committer = Committer::new(opts)?;
    committer.commit_all()?;
    println!("Successfully committed.");
    Ok(())
}
