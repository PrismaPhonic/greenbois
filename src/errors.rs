/// Errors originating from git2::Repository struct methods, translated for friendly error handling within
/// the gitchain library.
#[derive(Debug, Fail)]
pub enum RepositoryError {
    #[fail(display = "Failed to retrieve signature from git repository.")]
    SignatureRetrievalError {},
    #[fail(display = "Repository is missing git users name. Have you set it up yet?")]
    NameRetrievalError {},
    #[fail(display = "Repository is missing git users email. Have you set it up yet?")]
    EmailRetrievalError {},
    #[fail(display = "Failed to retrieve working directory for that repository.")]
    WorkdirRetrievalError {},
    #[fail(display = "Failed to open repository at that path.")]
    OpenError {},
    #[fail(display = "Could not find the current repositories index.")]
    FindIndexError {},
    #[fail(display = "Failed to write tree.")]
    TreeWriteError {},
}

/// General IO errors.
#[derive(Debug, Fail)]
pub enum IoError {
    #[fail(display = "Failed to open stdin.")]
    StdinOpenError {},
    #[fail(display = "Failed to write to stdin.")]
    StdinWriteError {},
}

/// Errors within the mining process, and always originating from the Miner struct.
#[derive(Debug, Fail)]
pub enum MiningError {
    #[fail(display = "Failed to find a nonce that when hashed with the commit satisfied the prefix constraint.")]
    SolveError {},
}

/// These errors relate to running git terminal commands internally within the library, and handling errors passed back from git.
#[derive(Debug, Fail)]
pub enum GitTerminalError {
    #[fail(display = "Failed to reset head to new commit.")]
    ResetHeadError {},
    #[fail(display = "Failed to generate commit object.")]
    CommitObjectError {},
    #[fail(display = "Failed to add files to staging.")]
    AddError {},
}

/// Errors from the Writer module.
#[derive(Debug, Fail)]
pub enum WriterErrors {
    #[fail(display = "Failed to format timestamp.")]
    TimeFormatError {},
}
