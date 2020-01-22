use failure::Error;

// time_str takes a time::Tm object and turns it into a string formatted per git commit
// convention.
pub fn time_string(time: time::OffsetDateTime) -> Result<String, Error> {
    Ok(
        format!(
            "{} {}",
            time.timestamp(),
            time
                .format("%z"),
        )
    )
}

pub fn generate_initial_blob(
    tree: &String,
    author: &String,
    message: &String,
    commit_time: time::OffsetDateTime,
) -> Result<String, Error> {
    let time_str = time_string(commit_time)?;

    Ok(
        format!(
            "tree {}\n\
         author {} {}\n\
         committer {} {}\n\n\
         {}",
            tree, author, time_str, author, time_str, message
        )
    )
}

pub fn generate_non_initial_blob(
    tree: &String,
    parent: &String,
    author: &String,
    message: &String,
    commit_time: time::OffsetDateTime,
) -> Result<String, Error> {
    let time_str = time_string(commit_time)?;

    Ok(
        format!(
            "tree {}\n\
             parent {}\n\
             author {} {}\n\
             committer {} {}\n\n\
             {}",
            tree, parent, author, time_str, author, time_str, message
        )
    )
}

/// Prepends the necessary header to the blob, which is necessary before we check the blobs
/// resulting hash, or the hash will be incorrect.
pub fn prepend_header_to_blob(blob: &str) -> Vec<u8> {
    format!("commit {}\0{}", blob.len(), blob).into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepending_header() {
        let blob = prepend_header_to_blob("test");
        assert_eq!(std::str::from_utf8(&blob).unwrap(), "commit 20\0test");
    }

    #[test]
    fn test_blob_generation_with_parent() -> Result<(), Error> {
        let tree = "TreeTest".to_string();
        let parent = Some("ParentTest".to_string());
        let author = "AuthorTest <test@test.com>".to_string();
        let message = "MessageTest".to_string();
        let commit_time = time::strptime("2016-02-05 16:52:22", "%Y-%m-%d %H:%M:%S")?;
        let blob = generate_blob(tree, parent, author, message, commit_time)?;

        let expected = "tree TreeTest\n\
                        parent ParentTest\n\
                        author AuthorTest <test@test.com> 1454691142 -0000\n\
                        committer AuthorTest <test@test.com> 1454691142 -0000\n\n\
                        MessageTest                               \n";
        assert_eq!(blob, expected);

        Ok(())
    }

    #[test]
    fn test_blob_generation_without_parent() -> Result<(), Error> {
        let tree = "TreeTest".to_string();
        let parent = None;
        let author = "AuthorTest <test@test.com>".to_string();
        let message = "MessageTest".to_string();
        let commit_time = time::strptime("2016-02-05 16:52:22", "%Y-%m-%d %H:%M:%S")?;
        let blob = generate_blob(tree, parent, author, message, commit_time)?;

        let expected = "tree TreeTest\n\
                        author AuthorTest <test@test.com> 1454691142 -0000\n\
                        committer AuthorTest <test@test.com> 1454691142 -0000\n\n\
                        MessageTest                                                 \n";
        assert_eq!(blob, expected);

        Ok(())
    }
}
