pub use crate::{
    nonce::NonceError,
    quiz::QuizError,
    vote_response::VoteResponseError,
};

pub type PollResult<T> = Result<T, PollError>;

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
