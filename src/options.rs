extern crate structopt;
use structopt::StructOpt;
use std::path::PathBuf;
use chrono::NaiveTime;

#[derive(StructOpt, Debug)]
#[structopt(
name = "greenbois",
about = "The green boi magnet",
)]
/// You can use gitchain to create a git commit with a git hash that is prefixed with zeros.
pub enum Opts {
    #[structopt(name = "commit")]
    /// Spawns commits starting from the date supplied to present day.
    Commit {
        /// Provide a path to the base directory of your github repository.
        #[structopt(short = "r", long = "repository", parse(from_os_str), default_value = ".")]
        repo: PathBuf,

        /// Message flag allows you to provide a commit message.
        #[structopt(short = "m", long = "message")]
        msg: String,

        /// Provide a number of years ago to begin writing the init commit from.
        /// Accepts floating point (e.g. 1.5 years ago)
        #[structopt(short = "y", long = "years", default_value = "1.0")]
        yrs_ago: f64,

        /// Provide a start time for your work day as an hour in 24 hour format.
        /// Example: 8 for 8am, or 10 for 10am.
        /// Defaults to 10.
        #[structopt(short = "s", long = "start", default_value = "10")]
        start: u32,

        /// Provide an end time for your work day as an hour in 24 hour format.
        /// Example: 17 for 5pm, or 20 for 8pm.
        /// Defaults to 20.
        #[structopt(short = "e", long = "end", default_value = "20")]
        end: u32,
    },
}

/// For ease of use in the application as a translation from commands
pub struct Options {
    pub repo: PathBuf,
    pub msg: String,
    pub yrs_ago: f64,
    pub start: NaiveTime,
    pub end: NaiveTime,
}
