pub mod types;

pub use crate::types::{
    quiz::{
        Quiz,
        QuizAnswer,
    },
    HtmlResponse,
    VoteResponse,
};
use rand::seq::IteratorRandom;
use rapidus::{
    parser::Parser,
    vm::{
        jsvalue::{
            array::ArrayObjectInfo,
            object::ObjectKind,
            value::Value as JsValue,
        },
        vm::VM,
    },
};
use reqwest::{
    header::{
        REFERER,
        USER_AGENT,
    },
    Url,
};
use select::{
    document::Document,
    predicate::{
        And,
        Attr,
        Name,
    },
};
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

pub const AGENTS: &str = include_str!("user-agents.txt");

pub type PollResult<T> = Result<T, PollError>;

#[derive(Debug)]
pub enum PollError {
    Network,
    InvalidBody,
}

fn get_time_ms() -> u128 {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH).unwrap().as_millis()
}

fn get_array_ref(val: &JsValue) -> Option<&ArrayObjectInfo> {
    match val {
        JsValue::Object(obj) => {
            let obj = unsafe { &**obj };
            match &obj.kind {
                ObjectKind::Array(a) => Some(&a),
                _ => None,
            }
        }
        _ => None,
    }
}

fn get_global(vm: &VM, val: &str) -> Option<JsValue> {
    vm.current_context.variable_environment.get_value(val).ok()
}

pub struct Client {
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Client {
            client: reqwest::Client::builder()
                .cookie_store(false)
                .build()
                .unwrap(),
        }
    }

    fn get_agent() -> &'static str {
        AGENTS.lines().choose(&mut rand::thread_rng()).unwrap()
    }

    fn get_poll_code(&self, agent: &str, quiz: &Quiz) -> PollResult<PollCode> {
        let url = format!(
            "https://polldaddy.com/n/{hash}/{id}?{time}",
            hash = quiz.get_hash(),
            id = quiz.get_id(),
            time = get_time_ms()
        );

        let mut res = self
            .client
            .get(&url)
            .header(USER_AGENT, agent)
            .header(REFERER, quiz.get_referer())
            .send()
            .map_err(|_| PollError::Network)?;

        if !res.status().is_success() {
            return Err(PollError::Network);
        }

        let data = res.text().map_err(|_| PollError::InvalidBody)?;
        PollCode::from_script_data(&data, quiz).ok_or(PollError::InvalidBody)
    }

    pub fn vote(&self, quiz: &Quiz, choice_index: usize) -> PollResult<VoteResponse> {
        let choice = quiz.get_answers()[choice_index].get_id();
        let agent = Self::get_agent();
        let code = self.get_poll_code(agent, &quiz)?;

        let url = format!(
            "https://polls.polldaddy.com/vote-js.php?p={}&b=1&a={},&o=&va={}&cookie=0&n={}&url={referer}",
            quiz.get_id(), choice, quiz.get_va(), code.0, referer = quiz.get_referer()
        );

        let mut res = self
            .client
            .get(&url)
            .header(USER_AGENT, agent)
            .header(REFERER, quiz.get_referer())
            .send()
            .map_err(|_| PollError::Network)?;

        if !res.status().is_success() {
            return Err(PollError::Network);
        }

        let data = res.text().map_err(|_| PollError::InvalidBody)?;
        VoteResponse::parse_response(&data, quiz).ok_or(PollError::InvalidBody)
    }

    pub fn quiz_from_url(&self, referer: &str) -> PollResult<Vec<PollResult<Quiz>>> {
        let res = self
            .client
            .get(referer)
            .send()
            .map_err(|_| PollError::Network)?;

        let doc = Document::from_read(res).map_err(|_| PollError::InvalidBody)?;
        Ok(doc
            .find(And(Name("script"), Attr("src", ())))
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
            .map(|(url, id)| {
                let mut res = self
                    .client
                    .get(url.as_str())
                    .send()
                    .map_err(|_| PollError::Network)?;

                if !res.status().is_success() {
                    return Err(PollError::Network);
                }

                let body = res.text().map_err(|_| PollError::InvalidBody)?;
                Quiz::from_script_data(referer.to_string(), id, &body).ok_or(PollError::InvalidBody)
            })
            .collect())
    }
}

#[derive(Debug)]
struct PollCode(String);

impl PollCode {
    fn from_script_data(data: &str, quiz: &Quiz) -> Option<PollCode> {
        let data = format!(
            r#"
var PD_vote{} = function(){{
		
}}
{}
"#,
            quiz.get_id(),
            data
        );
        let mut vm = VM::new();
        let mut parser = Parser::new("main", data);
        let node = parser.parse_all().ok()?;
        let func_info = vm.compile(&node, true).ok()?;
        vm.run_global(func_info).ok()?;
        let code = get_global(&vm, &format!("PDV_n{}", quiz.get_id()))?.to_string();

        Some(PollCode(code))
    }
}

/*
match e.kind {
    rapidus::vm::error::ErrorKind::Exception(v) => {
        dbg!(v.get_property("message").to_string());
    },
    _=> {

    },
}
*/
