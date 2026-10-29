use crate::committer::Committer;
use crate::dates;
use crate::errors::{Error, GitTerminalError, RepositoryError, RewriteError};
use chrono::{DateTime, Duration, Local};
use git2::{Oid, Repository, Signature, Sort, Time};
use rand::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// A Rewriter does the work of rewriting the history of an existing repository.
pub struct Rewriter {
    repo: Repository,
}

impl Rewriter {
    /// Creates a new Rewriter for the repository at the given path.
    pub fn new(repo: &PathBuf) -> Result<Rewriter, Error> {
        let repo = Repository::open(repo).map_err(|_| RepositoryError::OpenError {})?;

        Ok(Rewriter { repo })
    }

    pub fn rewrite_repo(
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

        let earliest = Rewriter::earliest_start(commits.len(), end_time);
        if start_time < earliest {
            return Err(RewriteError::StartTooEarly {
                earliest: earliest.date_naive(),
            }
            .into());
        }

        let days = Rewriter::allocate_to_days(commits.len(), start_time, end_time);

        let mut commit_times = Vec::new();
        for (day, count) in days {
            let times = Rewriter::spread_times(count, day, work_duration);
            commit_times.extend(times);
        }

        let mut rewritten = HashMap::new();
        for i in 0..commits.len() {
            let new_oid = self.rewrite_commit(commits[i], commit_times[i], &rewritten)?;
            rewritten.insert(commits[i], new_oid);
        }

        // HEAD is always the last commit when walking oldest first.
        let head = commits[commits.len() - 1];
        Committer::reset_head_to_hash(&self.repo, rewritten[&head])?;

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

        let author = Rewriter::signature_at(&old_commit.author(), commit_time)?;
        let committer = Rewriter::signature_at(&old_commit.committer(), commit_time)?;

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
}
