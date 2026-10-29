use crate::dates;
use crate::errors::{Error, GitTerminalError, RepositoryError, RewriteError};
use crate::options::Options;
use crate::writer;
use chrono::{DateTime, Duration, Local, NaiveTime};
use git2::ObjectType::Commit;
use git2::ResetType::Mixed;
use git2::{Oid, Repository, Signature, Sort, Time};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

const DAILY_COMMIT_CHOICES: [u8; 4] = [0, 1, 2, 3];
const DAILY_COMMIT_WEIGHTS: [u8; 4] = [1, 5, 2, 1];

/// A Committer does the work of issuing git commits.
pub struct Committer {
    tree: String,
    author: String,
    message: String,
    days_to_commit: i64,
    start_datetime: DateTime<Local>,
    start_hour: NaiveTime,
    end_hour: NaiveTime,
    repo: Repository,
}

impl Committer {
    /// Creates a new Committer.
    pub fn new(options: Options) -> Result<Committer, Error> {
        let mut repo = Committer::get_repository(&options.repo)?;
        let tree = Committer::create_tree(&mut repo)?;
        let author = Committer::get_author(&repo)?;

        let days_to_commit = (365.0 * options.yrs_ago).round() as i64;
        let now = Local::now();
        let corrected_now = now - (now.time() - options.start);
        let start_datetime = corrected_now - Duration::days(days_to_commit);

        return Ok(Committer {
            tree,
            author,
            message: options.msg,
            days_to_commit,
            start_datetime,
            start_hour: options.start,
            end_hour: options.end,
            repo,
        });
    }

    /// This method can be called to write all commits from yrs ago to current date.
    pub fn commit_all(&self) -> Result<(), Error> {
        // Write init commit.
        let mut commit_time = self.start_datetime;
        let mut blob =
            writer::generate_initial_blob(&self.tree, &self.author, &self.message, &commit_time)?;
        let mut parent = self.commit_blob(blob.clone().into_bytes())?;
        let work_duration = self.end_hour - self.start_hour;

        // Main loop to write commits up until present day.
        for _ in 1..self.days_to_commit {
            commit_time += Duration::days(1);
            let (p, b) = self.commit_from_time(&parent, &blob, commit_time, work_duration)?;
            parent = p;
            blob = b;
        }

        // Reset head at end.
        self.reset_head_to_hash(parent)?;

        Ok(())
    }

    fn rewrite_repo(
        &self,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
        work_duration: Duration,
    ) -> Result<(), Error> {
        // First we take the total number of commits in the tree already, and
        // the start and end time, and then we allocate commits to days in a
        // somewhat random spread until the commits have been allocated into
        // each "bucket" (day) accordingly
        let commits = self.collect_commits()?;

        let earliest = Committer::earliest_start(commits.len(), end_time);
        if start_time < earliest {
            return Err(RewriteError::StartTooEarly {
                earliest: earliest.date_naive(),
            }
            .into());
        }

        let days = Committer::allocate_to_days(commits.len(), start_time, end_time);

        let mut commit_times = Vec::new();
        for (day, count) in days {
            let times = Committer::spread_times(count, day, work_duration);
            commit_times.extend(times);
        }

        let mut rewritten = HashMap::new();
        for i in 0..commits.len() {
            let new_oid = self.rewrite_commit(commits[i], commit_times[i], &rewritten)?;
            rewritten.insert(commits[i], new_oid);
        }

        // HEAD is always the last commit when walking oldest first.
        let head = commits[commits.len() - 1];
        self.reset_head_to_hash(rewritten[&head])?;

        Ok(())
    }

    /// Returns the ids of every commit reachable from HEAD, oldest first.
    fn collect_commits(&self) -> Result<Vec<Oid>, Error> {
        let mut revwalk = self
            .repo
            .revwalk()
            .map_err(|_| RepositoryError::RevwalkError {})?;

        revwalk
            .push_head()
            .map_err(|_| RepositoryError::RevwalkError {})?;

        // Topological ensures parents always come before their children once reversed.
        revwalk
            .set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)
            .map_err(|_| RepositoryError::RevwalkError {})?;

        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid.map_err(|_| RepositoryError::RevwalkError {})?;
            commits.push(oid);
        }

        Ok(commits)
    }

    fn commit_from_time(
        &self,
        parent: &Oid,
        // TODO: This is very weird but required based on how I originally built it. Refactor.
        blob: &String,
        start_time: DateTime<Local>,
        work_duration: Duration,
    ) -> Result<(Oid, String), Error> {
        let num_of_commits = Committer::gen_rand_num_commits();
        let mut parent = *parent;
        let mut blob = blob.clone();

        for i in 0..num_of_commits {
            let commit_time = start_time
                + Duration::seconds(
                    ((work_duration.num_seconds() as f64 / num_of_commits as f64) * (i as f64))
                        as i64,
                );

            if dates::should_skip_date(&commit_time) {
                continue;
            }
            blob = writer::generate_non_initial_blob(
                &self.tree,
                parent.to_string(),
                &self.author,
                &self.message,
                &commit_time,
            )?;
            // TODO: This is wildly inefficient - fix.
            parent = self.commit_blob(blob.to_owned().into_bytes())?;
        }

        Ok((parent, blob.to_owned()))
    }

    pub fn gen_rand_num_commits() -> u8 {
        // Generate random number of times to commit today.
        // Weighted heavily towards a single commit, with occasional quiet or busy days.
        let dist = WeightedIndex::new(&DAILY_COMMIT_WEIGHTS).unwrap();
        let mut rng = rand::thread_rng();
        DAILY_COMMIT_CHOICES[dist.sample(&mut rng)]
    }

    /// The average number of commits per day that gen_rand_num_commits will produce.
    fn avg_daily_commits() -> f64 {
        let mut total_commits: u32 = 0;
        let mut total_weight: u32 = 0;
        for i in 0..DAILY_COMMIT_CHOICES.len() {
            total_commits += DAILY_COMMIT_CHOICES[i] as u32 * DAILY_COMMIT_WEIGHTS[i] as u32;
            total_weight += DAILY_COMMIT_WEIGHTS[i] as u32;
        }
        total_commits as f64 / total_weight as f64
    }

    /// Returns the earliest start date that commit_count commits can be spread across
    /// up until end_time (inclusive), based on the average number of commits per workday.
    fn earliest_start(commit_count: usize, end_time: DateTime<Local>) -> DateTime<Local> {
        let needed_workdays = (commit_count as f64 / Committer::avg_daily_commits()).ceil() as i64;

        let mut day = end_time;
        let mut workdays = 0;
        loop {
            if !dates::should_skip_date(&day) {
                workdays += 1;
            }
            if workdays >= needed_workdays {
                return day;
            }
            day -= Duration::days(1);
        }
    }

    /// Allocates commit_count commits across the workdays between start_time and end_time
    /// (inclusive), returning each workday alongside how many commits it was given.
    /// Any commits left over once we reach end_time are piled onto the last day.
    fn allocate_to_days(
        commit_count: usize,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
    ) -> Vec<(DateTime<Local>, usize)> {
        let mut days = Vec::new();
        let mut remaining = commit_count;
        let total_days = (end_time.date_naive() - start_time.date_naive()).num_days();

        for offset in 0..=total_days {
            if remaining == 0 {
                break;
            }

            let day = start_time + Duration::days(offset);
            if dates::should_skip_date(&day) {
                continue;
            }

            let mut count = Committer::gen_rand_num_commits() as usize;
            if count > remaining {
                count = remaining;
            }
            days.push((day, count));
            remaining -= count;
        }

        if remaining > 0 {
            if let Some(last_day) = days.last_mut() {
                last_day.1 += remaining;
            }
        }

        days
    }

    /// Picks count times spread across the work day beginning at day_start. The work day is
    /// split into count equal slots and each commit lands at a random point within its slot,
    /// so times stay in order without looking perfectly evenly spaced.
    fn spread_times(
        count: usize,
        day_start: DateTime<Local>,
        work_duration: Duration,
    ) -> Vec<DateTime<Local>> {
        let mut times = Vec::new();
        if count == 0 {
            return times;
        }

        let slot_seconds = work_duration.num_seconds() / count as i64;
        let mut rng = rand::thread_rng();
        for i in 0..count {
            let slot_start = slot_seconds * i as i64;
            let offset = rng.gen_range(slot_start, slot_start + slot_seconds);
            times.push(day_start + Duration::seconds(offset));
        }

        times
    }

    /// Copies the name and email from an existing signature, stamped with a new time.
    fn signature_at(
        original: &Signature,
        time: DateTime<Local>,
    ) -> Result<Signature<'static>, Error> {
        let name = original
            .name()
            .map_err(|_| RepositoryError::NameRetrievalError {})?;

        let email = original
            .email()
            .map_err(|_| RepositoryError::EmailRetrievalError {})?;

        let offset_minutes = time.offset().local_minus_utc() / 60;
        let git_time = Time::new(time.timestamp(), offset_minutes);

        let signature = Signature::new(name, email, &git_time)
            .map_err(|_| GitTerminalError::CommitObjectError {})?;

        Ok(signature)
    }

    /// Recreates an existing commit at a new time, with its parents swapped for their
    /// rewritten versions.
    fn rewrite_commit(
        &self,
        old_oid: Oid,
        commit_time: DateTime<Local>,
        rewritten: &HashMap<Oid, Oid>,
    ) -> Result<Oid, Error> {
        let old_commit = self
            .repo
            .find_commit(old_oid)
            .map_err(|_| RepositoryError::ReadCommitError {})?;

        let tree = old_commit
            .tree()
            .map_err(|_| RepositoryError::ReadCommitError {})?;

        let message = old_commit
            .message()
            .map_err(|_| RepositoryError::ReadCommitError {})?;

        let author = Committer::signature_at(&old_commit.author(), commit_time)?;
        let committer = Committer::signature_at(&old_commit.committer(), commit_time)?;

        // Commits are rewritten oldest first, so every parent has already been rewritten.
        let mut parents = Vec::new();
        for old_parent in old_commit.parent_ids() {
            let new_parent = rewritten[&old_parent];
            let parent = self
                .repo
                .find_commit(new_parent)
                .map_err(|_| RepositoryError::ReadCommitError {})?;
            parents.push(parent);
        }

        let new_oid = self
            .repo
            .commit(
                None,
                &author,
                &committer,
                message,
                &tree,
                &parents.iter().collect::<Vec<_>>(),
            )
            .map_err(|_| GitTerminalError::CommitObjectError {})?;

        Ok(new_oid)
    }

    fn reset_head_to_hash(&self, hash: Oid) -> Result<(), Error> {
        let obj = self
            .repo
            .find_object(hash, Some(Commit))
            .map_err(|_| GitTerminalError::ResetHeadError {})?;

        self.repo
            .reset(&obj, Mixed, None)
            .map_err(|_| GitTerminalError::ResetHeadError {})?;

        Ok(())
    }

    // Commits a blob returning the object id.
    fn commit_blob(&self, blob: Vec<u8>) -> Result<Oid, Error> {
        let oid = self
            .repo
            .odb()
            .map_err(|_| GitTerminalError::CommitObjectError {})?
            .write(Commit, &blob)
            .map_err(|_| GitTerminalError::CommitObjectError {})?;

        Ok(oid)
    }

    fn get_repository(repo: &PathBuf) -> Result<Repository, Error> {
        let repository = Repository::open(&repo).map_err(|_| RepositoryError::OpenError {})?;

        Ok(repository)
    }

    fn get_author(repo: &Repository) -> Result<String, Error> {
        let signature = repo
            .signature()
            .map_err(|_| RepositoryError::SignatureRetrievalError {})?;

        let name = signature
            .name()
            .map_err(|_| RepositoryError::NameRetrievalError {})?;

        let email = signature
            .email()
            .map_err(|_| RepositoryError::EmailRetrievalError {})?;

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
}
