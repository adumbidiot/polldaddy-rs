pub mod html_response;
use crate::{
    error::SendDuccError,
    util::JsEngine,
    vote_response::html_response::HtmlResponseError,
    HtmlResponse,
    Quiz,
};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
pub enum VoteResponseError {
    Json(serde_json::Error),
    Ducc(SendDuccError),
}

impl From<ducc::Error> for VoteResponseError {
    fn from(e: ducc::Error) -> Self {
        Self::Ducc(SendDuccError::from_ducc_error_lossy(e))
    }
}

impl From<serde_json::Error> for VoteResponseError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[derive(Debug)]
pub struct VoteResponse {
    json_response: Option<JsonResponse>,
    html_response: Result<HtmlResponse, HtmlResponseError>,
}

impl VoteResponse {
    pub fn parse_response(data: &str, quiz: &Quiz) -> Result<Self, VoteResponseError> {
        let vm = JsEngine::new()?;
        let patch = format!("var PD_button{id} = ''; var ret = null; var PDF_callback{id} = function(data){{ ret = data; }}", id = quiz.get_id());
        vm.exec(&patch)?;
        vm.exec(&data)?;

        let json_response = match vm.get_global::<_, Option<String>>("ret")? {
            Some(s) => serde_json::from_str(&s)?,
            None => None,
        };

        let html: String = vm
            .get_global::<_, ducc::Object>("document")?
            .call_prop::<_, _, ducc::Object>(
                "getElementById",
                (format!("PDI_container{}", quiz.get_id()),),
            )?
            .get("innerHTML")?;
        let html_response = HtmlResponse::from_doc_str(&html);

        Ok(VoteResponse {
            json_response,
            html_response,
        })
    }

    pub fn is_ip_banned(&self) -> bool {
        self.json_response.is_none() // Json response is absent if ip banned
    }

    pub fn registered_vote(&self) -> bool {
        self.json().map_or(false, |r| r.is_registered()) // Vote does not register if ip banned
    }

    pub fn html(&self) -> Result<&HtmlResponse, &HtmlResponseError> {
        self.html_response.as_ref()
    }

    pub fn json(&self) -> Option<&JsonResponse> {
        self.json_response.as_ref()
    }
}

#[derive(Debug, Deserialize)]
pub struct JsonResponse {
    id: Option<u64>,
    other_answer: Option<String>,
    result: Option<String>,
    answer: Vec<u64>,

    #[serde(flatten)]
    pub unknown: HashMap<String, serde_json::Value>,
}

impl JsonResponse {
    pub fn is_registered(&self) -> bool {
        self.result.as_ref().map_or(false, |el| el == "registered")
    }
}
