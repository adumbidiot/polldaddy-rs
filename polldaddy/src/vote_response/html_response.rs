use select::{
    document::Document,
    node::Node,
    predicate::{
        Class,
        Name,
        Text,
    },
};

#[derive(Debug)]
pub enum HtmlResponseError {
    MissingTotalVotes,
    InvalidTotalVotes(std::num::ParseIntError),
    MissingAnswers,
}

#[derive(Debug)]
pub struct HtmlResponse {
    answers: Vec<Result<AnswerResponse, AnswerResponseError>>,
    total_votes: u64,
}

impl HtmlResponse {
    pub fn get_total_votes(&self) -> u64 {
        self.total_votes
    }

    pub fn get_answers(&self) -> &[Result<AnswerResponse, AnswerResponseError>] {
        &self.answers
    }

    pub fn from_doc_str(data: &str) -> Result<Self, HtmlResponseError> {
        Self::from_doc(&Document::from(data))
    }

    pub fn from_doc(doc: &Document) -> Result<Self, HtmlResponseError> {
        let total_votes = doc
            .find(Class("pds-total-votes"))
            .last()
            .ok_or(HtmlResponseError::MissingTotalVotes)?
            .find(Name("span"))
            .last()
            .ok_or(HtmlResponseError::MissingTotalVotes)?
            .find(Text)
            .last()
            .ok_or(HtmlResponseError::MissingTotalVotes)?
            .as_text()
            .ok_or(HtmlResponseError::MissingTotalVotes)?
            .chars()
            .filter(|c| char::is_numeric(*c))
            .collect::<String>()
            .parse()
            .map_err(HtmlResponseError::InvalidTotalVotes)?;

        let answers = doc
            .find(Class("pds-answer"))
            .last()
            .ok_or(HtmlResponseError::MissingAnswers)?
            .find(Class("pds-feedback-group"))
            .map(AnswerResponse::from_node)
            .collect();

        Ok(HtmlResponse {
            total_votes,
            answers,
        })
    }
}

#[derive(Debug)]
pub enum AnswerResponseError {
    MissingAnswerText,
    MissingPercent,
    InvalidPercent(std::num::ParseFloatError),
    MissingAnswerVotes,
    InvalidAnswerVotes(std::num::ParseIntError),
}

#[derive(Debug)]
pub struct AnswerResponse {
    text: String,
    percent: f32,
    votes: u64,
}

impl AnswerResponse {
    fn from_node(el: Node) -> Result<Self, AnswerResponseError> {
        let text = get_text_from_node(el)?;
        let percent = get_percent_from_node(el)?;
        let votes = get_votes_from_node(el)?;

        Ok(AnswerResponse {
            text,
            percent,
            votes,
        })
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn get_votes(&self) -> u64 {
        self.votes
    }

    pub fn get_percent(&self) -> f32 {
        self.percent
    }
}

fn get_text_from_node(el: Node) -> Result<String, AnswerResponseError> {
    Ok(el
        .find(Class("pds-answer-text"))
        .last()
        .ok_or(AnswerResponseError::MissingAnswerText)?
        .find(Text)
        .last()
        .ok_or(AnswerResponseError::MissingAnswerText)?
        .as_text()
        .ok_or(AnswerResponseError::MissingAnswerText)?
        .trim()
        .to_string())
}

fn get_percent_from_node(el: Node) -> Result<f32, AnswerResponseError> {
    el.find(Class("pds-feedback-per"))
        .last()
        .ok_or(AnswerResponseError::MissingPercent)?
        .find(Text)
        .last()
        .ok_or(AnswerResponseError::MissingPercent)?
        .as_text()
        .ok_or(AnswerResponseError::MissingPercent)?
        .trim()
        .trim_end_matches('%')
        .parse()
        .map_err(AnswerResponseError::InvalidPercent)
}

fn get_votes_from_node(el: Node) -> Result<u64, AnswerResponseError> {
    el.find(Class("pds-feedback-votes"))
        .last()
        .ok_or(AnswerResponseError::MissingAnswerVotes)?
        .find(Text)
        .last()
        .ok_or(AnswerResponseError::MissingAnswerVotes)?
        .as_text()
        .ok_or(AnswerResponseError::MissingAnswerVotes)?
        .chars()
        .filter(|c| char::is_numeric(*c))
        .collect::<String>()
        .parse()
        .map_err(AnswerResponseError::InvalidAnswerVotes)
}

//https://sacbee.com/sports/high-school/article238211104.html
