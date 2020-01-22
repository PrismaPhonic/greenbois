use crate::errors::{GitTerminalError, IoError, RepositoryError};
use crate::options::Options;
use crate::writer;
use failure::Error;
use git2::Repository;
use std::env;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use time::{Duration, Time, PrimitiveDateTime, OffsetDateTime, Date};
use time::Weekday::{Saturday, Sunday};
use crate::hasher;
use rand::prelude::*;
use rand::distributions::WeightedIndex;

// TODO: Move these gross constants to not be here. Hacky for now
const NYDAY: Date = date!(2019-01-01);
const MEMORIAL_DAY: Date = date!(2019-05-27);
const INDEPENDENCE_DAY: Date = date!(2019-07-04);
const DAY_AFTER_INDEP: Date = date!(2019-07-05);
const LABOR_DAY: Date = date!(2019-09-02);
const VETERANS_DAY: Date = date!(2019-11-11);
const THANKSGIVING: Date = date!(2019-11-28);
const DAY_AFTER_THANX: Date = date!(2019-11-28);
const CHRISTMAS_EVE: Date = date!(2019-12-24);
const CHRISTMAS_DAY: Date = date!(2019-12-25);
const NYE: Date = date!(2019-12-31);
const HOLIDAYS: [Date; 11] = [NYDAY, MEMORIAL_DAY, INDEPENDENCE_DAY, DAY_AFTER_INDEP, LABOR_DAY, VETERANS_DAY, THANKSGIVING, DAY_AFTER_THANX, CHRISTMAS_EVE, CHRISTMAS_DAY, NYE];

fn is_holiday(date: Date) -> bool {
    for holiday in HOLIDAYS.iter() {
        if date.month_day() == holiday.month_day() {
            return true
        }
    }

    false
}

/// A Committer does the work of issuing git commits.
pub struct Committer {
    tree: String,
    parent: Option<String>,
    author: String,
    message: String,
    yrs_ago: f64,
    repo: PathBuf,
    working_dir: PathBuf,
}

struct CommitNode {
    tree: String,
    parent: Option<String>,
    author: String,
    working_dir: PathBuf,
}

impl CommitNode {
    pub fn get_current(repo: &PathBuf) -> Result<CommitNode, Error> {
        let mut repo = Committer::get_repository(repo)?;
        let working_dir = repo
            .workdir()
            .ok_or(RepositoryError::WorkdirRetrievalError {})?
            .to_path_buf();

        let tree = Committer::create_tree(&mut repo)?;
        let parent = Committer::get_parent(&repo);
        let author = Committer::get_author(&repo)?;

        Ok(CommitNode {
            tree,
            parent,
            author,
            working_dir,
        })
    }
}

impl Committer {
    /// Creates a new Committer.
    pub fn new(options: Options) -> Result<Committer, Error> {
        let CommitNode{tree, parent, author, working_dir} = CommitNode::get_current(&options.repo)?;

        return Ok(Committer {
            tree,
            parent,
            author,
            message: options.msg,
            yrs_ago: options.yrs_ago,
            repo: options.repo,
            working_dir,
        });
    }

    /// This method can be called to write all commits from yrs ago to current date.
    pub fn commit_all(&self) -> Result<(), Error> {
        // Write init commit.
        let days_to_commit = (365.0 * self.yrs_ago).round() as i64;
        let init_time = OffsetDateTime::now() - Duration::days(days_to_commit);
        let mut commit_time = init_time.clone();
        let mut blob = writer::generate_initial_blob(&self.tree, &self.author, &self.message, commit_time)?;
        self.commit_blob(blob.clone().into_bytes())?;
        let mut parent = hex::encode(hasher::hash_blob(&blob));

        // Main loop to write commits up until present day.
        for i in 1..days_to_commit {
            commit_time = init_time + Duration::days(i);
            // skip weekends and holidays.
            match commit_time.weekday() {
                Saturday | Sunday => continue,
                _ => (),
            }
            if is_holiday(commit_time.date()) {
                continue;
            }

            let (p, b) = self.commit(&parent, &blob, commit_time)?;
            parent = p;
            blob = b;
        }

        // Reset head at end.
        let final_hash = hasher::hash_blob(&blob);
        self.reset_head_to_hash(&final_hash)?;

        Ok(())
    }

    fn commit(&self, parent: &String, blob: &String, commit_time: OffsetDateTime) -> Result<(String, String), Error> {
        // Generate random number of times to commit today.
        // Weight upper and lower numbers more to create believable spread.
        let choices = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
        let weights = [3, 4, 2, 2, 2, 1, 1, 1, 1, 2, 2, 2, 4, 3];
        let mut dist = WeightedIndex::new(&weights).unwrap();
        let mut rng = rand::thread_rng();
        let num_to_commit = choices[dist.sample(&mut rng)];
        let mut parent = parent.clone();
        let mut blob = blob.clone();

        for i in 0..num_to_commit {
            let tm = commit_time + Duration::minutes(((1440.0 / num_to_commit as f64) * (i as f64)).round() as i64);
            blob = writer::generate_non_initial_blob(&self.tree, &parent, &self.author, &self.message, tm)?;
            self.commit_blob(blob.clone().into_bytes())?;
            parent = hex::encode(hasher::hash_blob(&blob));
        }

        Ok((parent, blob))
    }

    fn reset_head_to_hash(&self, hash: &[u8; 20]) -> Result<(), Error> {
        env::set_current_dir(&self.working_dir).unwrap();

        // TODO: Figure out how to hide stderr in the case that we actually need to retry.
        for _ in 0..5 {
            let mut command = Command::new("git")
                .args(&["reset", "--soft", &hex::encode(hash)])
                .spawn()
                .map_err(|_| GitTerminalError::ResetHeadError {})?;

            let status = command
                .wait()
                .map_err(|_| GitTerminalError::ResetHeadError {})?;

            if status.success() {
                break;
            }
        }

        Ok(())
    }

    fn commit_blob(&self, blob: Vec<u8>) -> Result<(), Error> {
        env::set_current_dir(&self.working_dir).unwrap();

        // TODO: Figure out how to hide stderr in the case that we actually need to retry.
        for _ in 0..5 {
            let mut commit_command = Command::new("git")
                .args(&["hash-object", "-t", "commit", "-w", "--stdin"])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .spawn()
                .map_err(|_| GitTerminalError::CommitObjectError {})?;

            let stdin = commit_command
                .stdin
                .as_mut()
                .ok_or(IoError::StdinOpenError {})?;

            stdin
                .write_all(blob.as_slice())
                .map_err(|_| IoError::StdinWriteError {})?;

            let status = commit_command
                .wait()
                .map_err(|_| IoError::StdinWriteError {})?;

            if status.success() {
                break;
            }
        }

        Ok(())
    }

    fn get_repository(repo: &PathBuf) -> Result<Repository, Error> {
        let repository =
            Repository::open(&repo).map_err(|_| RepositoryError::OpenError {})?;

        Ok(repository)
    }

    fn get_author(repo: &Repository) -> Result<String, Error> {
        let signature = repo
            .signature()
            .map_err(|_| RepositoryError::SignatureRetrievalError {})?;

        let name = signature
            .name()
            .ok_or(RepositoryError::NameRetrievalError {})?;

        let email = signature
            .email()
            .ok_or(RepositoryError::EmailRetrievalError {})?;

        Ok(format!("{} <{}>", name, email))
    }

    fn create_tree(repository: &mut Repository) -> Result<String, Error> {
        let mut index = repository
            .index()
            .map_err(|_| RepositoryError::FindIndexError {})?;

        let tree = index
            .write_tree()
            .map_err(|_| RepositoryError::TreeWriteError {})?;

        return Ok(format!("{}", tree));
    }

    fn get_parent(repository: &Repository) -> Option<String> {
        if let Ok(head) = repository.revparse_single("HEAD") {
            return Some(format!("{}", head.id()));
        } else {
            return None;
        }
    }
}
