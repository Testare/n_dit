// Should be level 0, like common.rs

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorLevel {
    NonCritical,
    Critical,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// For low-level failures, like the player accidentally tries to move a
    /// sprite into a wall.
    ///
    /// Might not be used: It might be better to expect the UI to
    /// not send the command in this case. In the case of CLI commands,
    /// even moving into a wall unsuccessfully might require a message.
    CommandNotSuccessful,
    /// For instance, MoveActiveCurio when there is no Node loaded
    InvalidForContext(String),
    /// For instance, MoveActiveCurio when there is no active curio
    NotPossibleForState(String),
    /// The program seems to be in an invalid state, but we can probably reverse
    /// to a stable state.
    FailureReversible(String),
    /// Basically, programmer messed up, and the program is now in an invalid
    /// state.
    /// Create a new save, do not overwrite old save, and dump state to a debug
    /// file.
    FailureCritical(String),
    LoadingError(LoadingError),
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

impl ToString for Error {
    fn to_string(&self) -> String {
        use Error::*;
        match self {
            CommandNotSuccessful => "Command unsucessful".to_string(),
            InvalidForContext(msg) => format!("Command not possible, requires context [{}]", msg),
            NotPossibleForState(msg) => format!("Command not currently possible [{}]", msg),
            FailureReversible(msg) => format!("Programmer error detected, aborting [{}]", msg),
            FailureCritical(msg) => format!("Programmer error detected, crashing [{}]", msg),
            LoadingError(err) => format!("Error occured while loading asset: [{:?}]", err),
            Io(err) => format!("IO [{}]", err),
            SerdeJson(err) => format!("Serde Json [{}]", err),
        }
    }
}

impl Error {
    pub fn error_level(&self) -> ErrorLevel {
        use Error::*;
        match self {
            FailureCritical(_) | FailureReversible(_) | LoadingError(_) => ErrorLevel::Critical,
            _ => ErrorLevel::NonCritical,
        }
    }

    pub fn is_critical(&self) -> bool {
        self.error_level() == ErrorLevel::Critical
    }
}

pub trait ErrorMsg {
    fn invalid<T>(&self) -> Result<T>;
    fn fail_reversible<T>(&self) -> Result<T>;
    fn fail_critical<T>(&self) -> Result<T>;
    fn invalid_msg(&self) -> Error;
    fn fail_reversible_msg(&self) -> Error;
    fn fail_critical_msg(&self) -> Error;
}

impl ErrorMsg for str {
    fn invalid<T>(&self) -> Result<T> {
        Err(Error::NotPossibleForState(self.to_string()))
    }
    fn fail_reversible<T>(&self) -> Result<T> {
        Err(Error::FailureReversible(self.to_string()))
    }
    fn fail_critical<T>(&self) -> Result<T> {
        Err(Error::FailureCritical(self.to_string()))
    }
    fn invalid_msg(&self) -> Error {
        Error::NotPossibleForState(self.to_string())
    }
    fn fail_reversible_msg(&self) -> Error {
        Error::FailureReversible(self.to_string())
    }
    fn fail_critical_msg(&self) -> Error {
        Error::FailureCritical(self.to_string())
    }
}

#[derive(Debug)]
pub enum LoadingError {
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    DecodeError(base64::DecodeError),
    MissingAsset(&'static str, String),
}

impl From<std::io::Error> for LoadingError {
    fn from(err: std::io::Error) -> Self {
        LoadingError::Io(err)
    }
}

impl From<serde_json::Error> for LoadingError {
    fn from(err: serde_json::Error) -> Self {
        LoadingError::SerdeJson(err)
    }
}

impl From<base64::DecodeError> for LoadingError {
    fn from(err: base64::DecodeError) -> Self {
        LoadingError::DecodeError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}
