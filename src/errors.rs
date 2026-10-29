/// Errors originating from git2::Repository struct methods, translated for friendly error handling within
/// the gitchain library.
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Failed to retrieve signature from git repository.")]
    SignatureRetrievalError {},
    #[error("Repository is missing git users name. Have you set it up yet?")]
    NameRetrievalError {},
    #[error("Repository is missing git users email. Have you set it up yet?")]
    EmailRetrievalError {},
    #[error("Failed to retrieve working directory for that repository.")]
    WorkdirRetrievalError {},
    #[error("Failed to open repository at that path.")]
    OpenError {},
    #[error("Could not find the current repositories index.")]
    FindIndexError {},
    #[error("Failed to write tree.")]
    TreeWriteError {},
}

/// General IO errors.
#[derive(Debug, thiserror::Error)]
pub enum IoError {
    #[error("Failed to open stdin.")]
    StdinOpenError {},
    #[error("Failed to write to stdin.")]
    StdinWriteError {},
}

/// These errors relate to running git terminal commands internally within the library, and handling errors passed back from git.
#[derive(Debug, thiserror::Error)]
pub enum GitTerminalError {
    #[error("Failed to reset head to new commit.")]
    ResetHeadError {},
    #[error("Failed to generate commit object.")]
    CommitObjectError {},
    #[error("Failed to add files to staging.")]
    AddError {},
}

/// Errors from the Writer module.
#[derive(Debug, thiserror::Error)]
pub enum WriterErrors {
    #[error("Failed to format timestamp.")]
    TimeFormatError {},
}

/// Top level error type wrapping all in-house errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Io(#[from] IoError),
    #[error(transparent)]
    GitTerminal(#[from] GitTerminalError),
    #[error(transparent)]
    Writer(#[from] WriterErrors),
}
