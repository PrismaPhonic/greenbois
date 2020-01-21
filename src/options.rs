extern crate structopt;
use structopt::StructOpt;
use std::path::PathBuf;

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
        #[structopt(short = "y", long = "years")]
        yrs_ago: f64,
    },
}

/// For ease of use in the application as a translation from commands
pub struct Options {
    pub repo: PathBuf,
    pub msg: String,
    pub yrs_ago: f64,
}
