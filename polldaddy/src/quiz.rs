use crate::{
    util::{
        get_time_ms,
        JsEngine,
    },
    Nonce,
};
use url::Url;

#[derive(Debug)]
pub enum QuizError {
    Ducc(ducc::Error),
    QuizAnswer(QuizAnswerError),
}

impl From<ducc::Error> for QuizError {
    fn from(e: ducc::Error) -> Self {
        Self::Ducc(e)
    }
}

impl From<QuizAnswerError> for QuizError {
    fn from(e: QuizAnswerError) -> Self {
        Self::QuizAnswer(e)
    }
}

#[derive(Debug, Clone)]
pub struct Quiz {
    id: u32,
    answers: Vec<QuizAnswer>,
    hash: String,
    closed: bool,
    referer: String,
    va: String, // I don't know what this is but i need it
}

impl Quiz {
    pub fn from_script_data(referer: String, id: u32, data: &str) -> Result<Self, QuizError> {
        let vm = JsEngine::new()?;
        vm.exec(data)?;

        let hash = vm.get_global(format!("PDV_h{}", id))?;
        let closed = vm.get_global(format!("pollClosed{}", id))?;
        let va = vm.get_global(format!("PDV_va{}", id))?;
        let answers = vm
            .get_global::<_, Vec<Vec<String>>>(format!("PDV_A{}", id))?
            .iter()
            .map(|a| QuizAnswer::from_string_array(a))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Quiz {
            id,
            answers,
            hash,
            closed,
            referer,
            va,
        })
    }

    pub fn get_code_url(&self) -> Result<Url, url::ParseError> {
        let url_str = format!(
            "https://polldaddy.com/n/{hash}/{id}?{time}",
            hash = self.hash,
            id = self.id,
            time = get_time_ms()
        );

        Url::parse(&url_str)
    }

    pub fn get_vote_url(&self, answer: u32, nonce: &Nonce) -> Result<Url, url::ParseError> {
        let url_str = format!(
            "https://polls.polldaddy.com/vote-js.php?p={id}&b=1&a={answer},&o=&va={va}&cookie=0&n={nonce}&url={referer}",
            id = self.id, answer = answer, va = self.va, nonce = nonce.as_str(), referer = self.referer
        );
        Url::parse(&url_str)
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn is_closed(&self) -> bool {
        self.closed
    }

    pub fn get_referer(&self) -> &str {
        &self.referer
    }

    pub fn get_answers(&self) -> &[QuizAnswer] {
        &self.answers
    }

    pub fn get_va(&self) -> &str {
        &self.va
    }
}

#[derive(Debug, Clone)]
pub enum QuizAnswerError {
    MissingString(usize),
    BadIdParse(std::num::ParseIntError),
}

#[derive(Debug, Clone)]
pub struct QuizAnswer {
    id: u32,
    text: String,
}

impl QuizAnswer {
    fn from_string_array(arr: &[String]) -> Result<Self, QuizAnswerError> {
        let mut iter = arr.iter();
        Ok(QuizAnswer {
            id: iter
                .next()
                .ok_or(QuizAnswerError::MissingString(0))?
                .parse()
                .map_err(QuizAnswerError::BadIdParse)?,
            text: String::from(iter.next().ok_or(QuizAnswerError::MissingString(1))?),
        })
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}
