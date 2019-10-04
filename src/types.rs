pub mod quiz;

use crate::{
    get_global,
    Quiz,
};
use rapidus::{
    parser::Parser,
    vm::{
        jsvalue::value::Value as JsValue,
        vm::VM,
    },
};
use select::{
    document::Document,
    predicate::{
        Class,
        Name,
        Text,
    },
};

#[derive(Debug)]
pub struct VoteResponse {
    pub data: serde_json::Value,

    pub html_response: Option<HtmlResponse>,
}

impl VoteResponse {
    pub fn parse_response(data: &str, quiz: &Quiz) -> Option<Self> {
        let data = format!(
            r#"
var ret = null;
var html = {{}};

var PD_button{id} = "";
var document = {{}};
document.getElementById = function(id){{
	if(html[id] == undefined) html[id] = {{}};
	return html[id];
}}
	
var PDF_callback{id} = function(data){{
	ret = data;
}}

var is_secure = function(){{
	return true;
}}

var encodeURIComponent = function(){{
	return "";
}}

var window = {{}};
window.location = {{}};
window.location.href = "";
{}
"#,
            data,
            id = quiz.get_id()
        );
        let mut vm = VM::new();
        let mut parser = Parser::new("main", data);
        let node = parser.parse_all().ok()?;
        let func_info = vm.compile(&node, true).ok()?;
        vm.run_global(func_info).ok()?;
        let val = get_global(&vm, "ret")?;
        let data = serde_json::from_str(&val.to_string()).ok()?;
        let html = get_global(&vm, "html")?
            .get_property(&format!("PDI_container{}", quiz.get_id()))
            .get_property("innerHTML");

        let html_response = match html {
            JsValue::String(_) => {
                let doc = Document::from(html.to_string().as_str());
                HtmlResponse::from_doc(&doc)
            }
            _ => None,
        };

        Some(VoteResponse {
            data,
            html_response,
        })
    }

    pub fn is_banned(&self) -> bool {
        self.data.is_null()
    }
}

#[derive(Debug)]
pub struct HtmlResponse {
    answers: Vec<Option<AnswerResponse>>,
    total_votes: u64,
}

impl HtmlResponse {
    pub fn get_total_votes(&self) -> u64 {
        self.total_votes
    }

    pub fn get_answers(&self) -> &[Option<AnswerResponse>] {
        &self.answers
    }

    pub fn from_doc(doc: &Document) -> Option<Self> {
        let total_votes = doc
            .find(Class("pds-total-votes"))
            .last()?
            .find(Name("span"))
            .last()?
            .find(Text)
            .last()?
            .as_text()?
            .chars()
            .filter(|c| char::is_numeric(*c))
            .collect::<String>()
            .parse()
            .ok()?;

        let answers = doc
            .find(Class("pds-answer"))
            .last()?
            .find(Class("pds-feedback-group"))
            .map(|el| {
                let text = el
                    .find(Class("pds-answer-text"))
                    .last()?
                    .find(Text)
                    .last()?
                    .as_text()?
                    .trim()
                    .to_string();

                let percent: f32 = el
                    .find(Class("pds-feedback-per"))
                    .last()?
                    .find(Text)
                    .last()?
                    .as_text()?
                    .trim()
                    .trim_end_matches('%')
                    .parse()
                    .ok()?;

                let votes: u64 = el
                    .find(Class("pds-feedback-votes"))
                    .last()?
                    .find(Text)
                    .last()?
                    .as_text()?
                    .chars()
                    .filter(|c| char::is_numeric(*c))
                    .collect::<String>()
                    .parse()
                    .ok()?;

                Some(AnswerResponse {
                    text,
                    percent,
                    votes,
                })
            })
            .collect();

        Some(HtmlResponse {
            total_votes,
            answers,
        })
    }
}

#[derive(Debug)]
pub struct AnswerResponse {
    text: String,
    percent: f32,
    votes: u64,
}

impl AnswerResponse {
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

#[derive(Debug)]
pub struct PollCode(String);

impl PollCode {
    pub fn from_script_data(data: &str, quiz: &Quiz) -> Option<PollCode> {
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

    pub fn get_string(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
