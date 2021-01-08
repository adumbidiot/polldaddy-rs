use crate::{
    Nonce,
    PollError,
    PollResult,
    Quiz,
    VoteResponse,
    USER_AGENTS_LIST,
};
use futures::stream::StreamExt;
use rand::seq::IteratorRandom;
use reqwest::header::{
    REFERER,
    USER_AGENT,
};
use select::{
    document::Document,
    predicate::{
        And,
        Attr,
        Name,
    },
};
use url::Url;

#[derive(Clone, Default)]
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_reqwest(client: reqwest::Client) -> Self {
        Client { client }
    }

    fn get_agent() -> &'static str {
        USER_AGENTS_LIST
            .lines()
            .choose(&mut rand::thread_rng())
            .unwrap()
    }

    async fn get_nonce(&self, agent: &str, quiz: &Quiz) -> PollResult<Nonce> {
        let url = quiz.get_code_url()?;

        let res = self
            .client
            .get(url.as_str())
            .header(USER_AGENT, agent)
            .header(REFERER, quiz.get_referer())
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(PollError::InvalidStatus(status));
        }

        let text = res.text().await?;
        Ok(Nonce::from_script_data(&text, quiz)?)
    }

    pub async fn vote(&self, quiz: &Quiz, choice_index: usize) -> PollResult<VoteResponse> {
        let choice = quiz
            .get_answers()
            .get(choice_index)
            .ok_or(PollError::InvalidChoice(choice_index))?
            .get_id();

        let agent = Self::get_agent();
        let code = self.get_nonce(agent, &quiz).await?;
        let url = quiz.get_vote_url(choice, &code)?;

        let res = self
            .client
            .get(url.as_str())
            .header(USER_AGENT, agent)
            .header(REFERER, quiz.get_referer())
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(PollError::InvalidStatus(status));
        }

        let data = res.text().await?;
        Ok(VoteResponse::parse_response(&data, quiz)?)
    }

    pub async fn quiz_from_url(&self, referer: &str) -> PollResult<Vec<PollResult<Quiz>>> {
        let res = self.client.get(referer).send().await?; // Probably don't care if the status is invalid

        let text = res.text().await?;
        let script_filter = And(Name("script"), Attr("src", ()));
        let data_pairs: Vec<(Url, u32)> = {
            let doc = Document::from(text.as_str());

            doc.find(script_filter)
                .filter_map(|el| {
                    let url = el.attr("src").and_then(|src| Url::parse(src).ok())?;
                    if url.host_str()?.starts_with("secure.polldaddy.com") {
                        let id: u32 = url
                            .path_segments()?
                            .last()?
                            .trim_end_matches(".js")
                            .parse()
                            .ok()?;
                        Some((url, id))
                    } else {
                        None
                    }
                })
                .collect()
        };

        let ret = futures::stream::iter(data_pairs.into_iter())
            .then(|(url, id)| async move {
                let res = self.client.get(url.as_str()).send().await?;
                let status = res.status();
                if !status.is_success() {
                    return Err(PollError::InvalidStatus(status));
                }

                let body = res.text().await?;
                Ok(Quiz::from_script_data(String::from(referer), id, &body)?)
            })
            .collect()
            .await;

        Ok(ret)
    }
}
