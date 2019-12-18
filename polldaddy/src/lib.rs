pub mod client;
pub mod error;
pub mod nonce;
pub mod quiz;
pub mod util;
pub mod vote_response;

pub use crate::{
    client::Client,
    error::{
        PollError,
        PollResult,
    },
    nonce::Nonce,
    quiz::{
        Quiz,
        QuizAnswer,
    },
    vote_response::{
        html_response::HtmlResponse,
        JsonResponse,
        VoteResponse,
    },
};

pub const USER_AGENTS_LIST: &str = include_str!("user-agents.txt");

// Copied from reqwest
fn _assert_impls() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    fn assert_clone<T: Clone>() {}

    assert_send::<PollError>();
    assert_sync::<PollError>();
}
