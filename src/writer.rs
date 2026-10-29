use chrono::{DateTime, Local};
use crate::errors::Error;

// time_str takes a chrono::Local object and turns it into a string formatted per git commit
// convention.
pub fn time_string(time: &DateTime<Local>) -> Result<String, Error> {
    Ok(time.format("%s %z").to_string())
}

pub fn generate_initial_blob(
    tree: &str,
    author: &str,
    message: &str,
    commit_time: &DateTime<Local>,
) -> Result<String, Error> {
    let time_str = time_string(commit_time)?;

    Ok(format!(
        "tree {}\n\
         author {} {}\n\
         committer {} {}\n\n\
         {}",
        tree, author, time_str, author, time_str, message
    ))
}

pub fn generate_non_initial_blob(
    tree: &str,
    parent: String,
    author: &str,
    message: &str,
    commit_time: &DateTime<Local>,
) -> Result<String, Error> {
    let time_str = time_string(commit_time)?;

    Ok(format!(
        "tree {}\n\
             parent {}\n\
             author {} {}\n\
             committer {} {}\n\n\
             {}",
        tree, parent, author, time_str, author, time_str, message
    ))
}

/// Prepends the necessary header to the blob, which is necessary before we check the blobs
/// resulting hash, or the hash will be incorrect.
pub fn prepend_header_to_blob(blob: &str) -> Vec<u8> {
    format!("commit {}\0{}", blob.len(), blob).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_prepending_header() {
        let blob = prepend_header_to_blob("test");
        assert_eq!(std::str::from_utf8(&blob).unwrap(), "commit 4\0test");
    }

    #[test]
    fn test_time_string_offset_format() -> Result<(), Error> {
        let commit_time = Local.timestamp_opt(1454691142, 0).unwrap();
        let time_str = time_string(&commit_time)?;
        let (timestamp, offset) = time_str.split_once(' ').unwrap();

        // Git requires offsets formatted as +HHMM or -HHMM, with no colon.
        assert_eq!(timestamp, "1454691142");
        assert_eq!(offset.len(), 5);
        assert!(offset.starts_with('+') || offset.starts_with('-'));
        assert!(offset[1..].chars().all(|c| c.is_ascii_digit()));

        Ok(())
    }

    #[test]
    fn test_blob_generation_with_parent() -> Result<(), Error> {
        let tree = "TreeTest";
        let parent = "ParentTest".to_string();
        let author = "AuthorTest <test@test.com>";
        let message = "MessageTest";
        let commit_time = Local.timestamp_opt(1454691142, 0).unwrap();
        let blob = generate_non_initial_blob(tree, parent, author, message, &commit_time)?;

        let expected = format!(
            "tree TreeTest\n\
             parent ParentTest\n\
             author AuthorTest <test@test.com> 1454691142 {0}\n\
             committer AuthorTest <test@test.com> 1454691142 {0}\n\n\
             MessageTest",
            commit_time.format("%z")
        );
        assert_eq!(blob, expected);

        Ok(())
    }

    #[test]
    fn test_blob_generation_without_parent() -> Result<(), Error> {
        let tree = "TreeTest";
        let author = "AuthorTest <test@test.com>";
        let message = "MessageTest";
        let commit_time = Local.timestamp_opt(1454691142, 0).unwrap();
        let blob = generate_initial_blob(tree, author, message, &commit_time)?;

        let expected = format!(
            "tree TreeTest\n\
             author AuthorTest <test@test.com> 1454691142 {0}\n\
             committer AuthorTest <test@test.com> 1454691142 {0}\n\n\
             MessageTest",
            commit_time.format("%z")
        );
        assert_eq!(blob, expected);

        Ok(())
    }
}
