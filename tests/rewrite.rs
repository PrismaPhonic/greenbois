use chrono::{DateTime, Local, NaiveDate, NaiveTime, TimeZone, Timelike};
use git2::{Oid, Repository, Signature, Sort};
use greenbois::dates;
use greenbois::errors::{Error, RewriteError};
use greenbois::rewriter::Rewriter;
use std::process::Command;
use tempfile::TempDir;

const COMMIT_COUNT: usize = 8;

/// Creates a repository with COMMIT_COUNT real commits, each changing a file.
fn create_repo() -> (TempDir, Repository) {
    let dir = TempDir::new().unwrap();
    let repo = Repository::init(dir.path()).unwrap();
    let signature = Signature::now("Test Author", "test@test.com").unwrap();

    let mut parent: Option<Oid> = None;
    for i in 0..COMMIT_COUNT {
        let blob = repo.blob(format!("content {}", i).as_bytes()).unwrap();
        let mut builder = repo.treebuilder(None).unwrap();
        builder.insert("file.txt", blob, 0o100644).unwrap();
        let tree = repo.find_tree(builder.write().unwrap()).unwrap();

        let mut parents = Vec::new();
        if let Some(parent) = parent {
            parents.push(repo.find_commit(parent).unwrap());
        }

        let message = format!("Commit number {}", i);
        let oid = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                &message,
                &tree,
                &parents.iter().collect::<Vec<_>>(),
            )
            .unwrap();
        parent = Some(oid);
    }

    (dir, repo)
}

/// Returns every commit reachable from HEAD, oldest first.
fn history(repo: &Repository) -> Vec<Oid> {
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();
    revwalk
        .set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)
        .unwrap();

    let mut commits = Vec::new();
    for oid in revwalk {
        commits.push(oid.unwrap());
    }
    commits
}

fn local_time(date: NaiveDate, hour: u32) -> DateTime<Local> {
    date.and_time(NaiveTime::from_hms_opt(hour, 0, 0).unwrap())
        .and_local_timezone(Local)
        .single()
        .unwrap()
}

#[test]
fn test_rewrite_preserves_content_and_spreads_dates() {
    let (dir, repo) = create_repo();
    let original = history(&repo);

    // Monday to Friday, with no holidays or DST changes in between.
    let start_time = local_time(NaiveDate::from_ymd_opt(2024, 4, 8).unwrap(), 10);
    let end_time = local_time(NaiveDate::from_ymd_opt(2024, 4, 12).unwrap(), 10);
    let work_end = local_time(NaiveDate::from_ymd_opt(2024, 4, 12).unwrap(), 20);
    let work_duration = work_end - end_time;

    let rewriter = Rewriter::new(&dir.path().to_path_buf()).unwrap();
    rewriter
        .rewrite_repo(start_time, end_time, work_duration)
        .unwrap();

    let rewritten = history(&repo);
    assert_eq!(rewritten.len(), original.len());

    let mut previous_time = start_time;
    for i in 0..original.len() {
        let old_commit = repo.find_commit(original[i]).unwrap();
        let new_commit = repo.find_commit(rewritten[i]).unwrap();

        // Content and authorship are untouched.
        assert_ne!(old_commit.id(), new_commit.id());
        assert_eq!(old_commit.tree_id(), new_commit.tree_id());
        assert_eq!(old_commit.message(), new_commit.message());
        assert_eq!(old_commit.author().name(), new_commit.author().name());
        assert_eq!(old_commit.author().email(), new_commit.author().email());

        // Author and committer times match, and carry the local offset.
        let git_time = new_commit.time();
        assert_eq!(git_time, new_commit.author().when());
        let commit_time = Local.timestamp_opt(git_time.seconds(), 0).unwrap();
        assert_eq!(
            git_time.offset_minutes(),
            commit_time.offset().local_minus_utc() / 60
        );

        // Within the date range, within work hours, on a workday, and in order.
        assert!(commit_time >= start_time);
        assert!(commit_time.date_naive() <= end_time.date_naive());
        assert!(commit_time.hour() >= 10 && commit_time.hour() < 20);
        assert!(!dates::should_skip_date(&commit_time));
        assert!(commit_time >= previous_time);
        previous_time = commit_time;
    }

    // Git itself is happy with the rewritten objects.
    let fsck = Command::new("git")
        .args(&["fsck", "--strict"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    assert!(
        fsck.status.success(),
        "git fsck failed: {}",
        String::from_utf8_lossy(&fsck.stderr)
    );
}

#[test]
fn test_rewrite_rejects_start_too_early() {
    let (dir, repo) = create_repo();
    let original = history(&repo);

    // Far more workdays than 8 commits could believably fill.
    let start_time = local_time(NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(), 10);
    let end_time = local_time(NaiveDate::from_ymd_opt(2024, 4, 12).unwrap(), 10);
    let work_duration = chrono::Duration::hours(10);

    let rewriter = Rewriter::new(&dir.path().to_path_buf()).unwrap();
    let result = rewriter.rewrite_repo(start_time, end_time, work_duration);

    assert!(matches!(
        result,
        Err(Error::Rewrite(RewriteError::StartTooEarly { .. }))
    ));

    // History is left alone.
    assert_eq!(history(&repo), original);
}
