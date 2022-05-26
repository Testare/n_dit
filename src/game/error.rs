// Should be level 0, like common.rs

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum Error {
    #[deprecated]
    NodeActionError(String),

    /// For low-level failures, like the player accidentally tries to move a
    /// piece into a wall.
    ///
    /// Might not be used: It might be better to expect the UI to
    /// not send the command in this case. In the case of CLI commands,
    /// even moving into a wall unsuccessfully might require a message.
    CommandNotSuccessful,
    /// For instance, MoveActiveSprite when there is no Node loaded
    InvalidForContext(String),
    /// For instance, MoveActiveSprite when there is no active sprite
    NotPossibleForState(String),
    /// The program seems to be in an invalid state, but we can probably reverse
    /// to a stable state.
    FailureReversible(String),
    /// Basically, programmer messed up, and the program is now in an invalid
    /// state.
    /// Create a new save, do not overwrite old save, and dump state to a debug
    /// file.
    FailureCritical(String),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        match self {
            Error::NodeActionError(str) => str.to_owned(),
            _ => unimplemented!(""),
        }
    }
}
