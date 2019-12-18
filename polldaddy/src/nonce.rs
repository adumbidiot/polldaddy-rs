use crate::{
    util::JsEngine,
    Quiz,
};

#[derive(Debug)]
pub enum NonceError {
    Ducc(ducc::Error),
}

impl From<ducc::Error> for NonceError {
    fn from(e: ducc::Error) -> Self {
        Self::Ducc(e)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Nonce(String);

impl Nonce {
    pub fn from_script_data(data: &str, quiz: &Quiz) -> Result<Self, NonceError> {
        let vm = JsEngine::new()?;
        let vote_patch = format!("var PD_vote{} = function(){{}}", quiz.get_id());
        vm.exec(&vote_patch)?;
        vm.exec(&data)?;
        let code = vm.get_global(format!("PDV_n{}", quiz.get_id()))?;
        Ok(Nonce(code))
    }

    pub fn into_string(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
