
/// committer contains methods for issuing git commits.
pub mod committer;
/// dates contains constants of holidays and logic around which days and times to skip committing on.
pub mod dates;
/// custom in-house errors that we translate to from other errors received by external crates.
pub mod errors;
/// options contains Structopt enum for parsing terminal commands and providing helpful menus.
pub mod options;
/// rewriter contains methods for rewriting the history of an existing repository.
pub mod rewriter;
/// writer contains methods for building and manipulating git blobs.
pub mod writer;

use crate::committer::Committer;
pub use crate::options::{Options, Opts};
use crate::rewriter::Rewriter;

use chrono::{Local, NaiveDate, NaiveTime};
use std::path::PathBuf;
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
        Opts::Rewrite {
            repo,
            from,
            to,
            start,
            end,
        } => rewrite(repo, from, to, start, end),
    }
}

fn commit(opts: Options) -> Result<(), Error> {
    let committer = Committer::new(opts)?;
    committer.commit_all()?;
    println!("Successfully committed.");
    Ok(())
}

fn rewrite(
    repo: PathBuf,
    from: NaiveDate,
    to: Option<NaiveDate>,
    start: u32,
    end: u32,
) -> Result<(), Error> {
    // TODO: Deal with invalid times.
    let start_hour = NaiveTime::from_hms_opt(start, 0, 0).expect("Invalid time");
    let end_hour = NaiveTime::from_hms_opt(end, 0, 0).expect("Invalid time");
    let to = to.unwrap_or(Local::now().date_naive());

    // Both ends sit at the start of the work day, with commits spread forward from there.
    let start_time = from
        .and_time(start_hour)
        .and_local_timezone(Local)
        .single()
        .expect("Invalid local time");
    let end_time = to
        .and_time(start_hour)
        .and_local_timezone(Local)
        .single()
        .expect("Invalid local time");

    let rewriter = Rewriter::new(&repo)?;
    rewriter.rewrite_repo(start_time, end_time, end_hour - start_hour)?;
    println!("Successfully rewrote history.");
    Ok(())
}
