use crate::{
    get_array_ref,
    get_global,
};
use rapidus::{
    parser::Parser,
    vm::{
        jsvalue::value::Value as JsValue,
        vm::VM,
    },
};

#[derive(Debug)]
pub struct Quiz {
    id: u32,
    answers: Vec<QuizAnswer>,
    hash: String,
    closed: bool,
    referer: String,
    va: String, // I don't know what this is but i need it
}

impl Quiz {
    pub fn from_script_data(referer: String, id: u32, data: &str) -> Option<Self> {
        let data: String = data
            .lines()
            .filter(|line| !line.starts_with("var PDV_def"))
            .collect(); // Js Engine is REALLY buggy. Need to weed out line that crashes the parser.

        let mut vm = VM::new();
        let mut parser = Parser::new("main", &data);
        let node = parser.parse_all().ok()?;
        let func_info = vm.compile(&node, true).ok()?;
        vm.run_global(func_info).ok()?;

        let hash = get_global(&vm, &format!("PDV_h{}", id))?.to_string();
        let closed = get_global(&vm, &format!("pollClosed{}", id))?.into_bool();
        let va = get_global(&vm, &format!("PDV_va{}", id))?.to_string();
        let answers_val = get_global(&vm, &format!("PDV_A{}", id))?;
        let answers = get_array_ref(&answers_val)?
            .elems
            .iter()
            .map(|el| QuizAnswer::from_js_value(&el.get_data()?.val))
            .collect::<Option<Vec<_>>>()?;

        Some(Quiz {
            id,
            answers,
            hash,
            closed,
            referer,
            va,
        })
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

#[derive(Debug)]
pub struct QuizAnswer {
    id: u32,
    text: String,
}

impl QuizAnswer {
    fn from_js_value(val: &JsValue) -> Option<Self> {
        let mut iter = get_array_ref(&val)?
            .elems
            .iter()
            .map(|el| el.get_data().map(|data| data.val.to_string()));

        Some(QuizAnswer {
            id: iter.next()??.parse().ok()?,
            text: iter.next()??,
        })
    }

    pub fn get_id(&self) -> u32 {
        self.id
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}
