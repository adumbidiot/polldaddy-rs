pub use crate::{
    nonce::NonceError,
    quiz::QuizError,
    vote_response::VoteResponseError,
};

pub type PollResult<T> = Result<T, PollError>;

/// Ducc Error type with Send
#[derive(Debug)]
pub struct SendDuccError {
    pub kind: ErrorKind,
    pub context: Vec<String>,
}

impl SendDuccError {
    pub fn from_ducc_error_lossy(e: ducc::Error) -> Self {
        SendDuccError {
            kind: ErrorKind::from_ducc_error_kind_lossy(e.kind),
            context: e.context,
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    ToJsConversionError {
        from: &'static str,
        to: &'static str,
    },
    FromJsConversionError {
        from: &'static str,
        to: &'static str,
    },
    RuntimeError {
        code: ducc::RuntimeErrorCode,
        name: String,
    },
    RecursiveMutCallback,
    ExternalError,
    NotAFunction,
}

impl ErrorKind {
    pub fn from_ducc_error_kind_lossy(kind: ducc::ErrorKind) -> Self {
        match kind {
            ducc::ErrorKind::ToJsConversionError { from, to } => {
                ErrorKind::ToJsConversionError { from, to }
            }
            ducc::ErrorKind::FromJsConversionError { from, to } => {
                ErrorKind::FromJsConversionError { from, to }
            }
            ducc::ErrorKind::RuntimeError { code, name } => ErrorKind::RuntimeError { code, name },
            ducc::ErrorKind::RecursiveMutCallback => ErrorKind::RecursiveMutCallback,
            ducc::ErrorKind::ExternalError(_) => ErrorKind::ExternalError, // Data may or may not be Send. We must remove it to be Send.
            ducc::ErrorKind::NotAFunction => ErrorKind::NotAFunction,
        }
    }
}

#[derive(Debug)]
pub enum PollError {
    Reqwest(reqwest::Error),
    Io(std::io::Error),
    InvalidStatus(reqwest::StatusCode),
    InvalidQuiz(QuizError),
    Url(url::ParseError),
    InvalidChoice(usize),
    InvalidNonce(NonceError),
    InvalidVoteResponse(VoteResponseError),
}

impl From<reqwest::Error> for PollError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<std::io::Error> for PollError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<QuizError> for PollError {
    fn from(e: QuizError) -> Self {
        Self::InvalidQuiz(e)
    }
}

impl From<url::ParseError> for PollError {
    fn from(e: url::ParseError) -> Self {
        Self::Url(e)
    }
}

impl From<NonceError> for PollError {
    fn from(e: NonceError) -> Self {
        Self::InvalidNonce(e)
    }
}

impl From<VoteResponseError> for PollError {
    fn from(e: VoteResponseError) -> Self {
        Self::InvalidVoteResponse(e)
    }
}
