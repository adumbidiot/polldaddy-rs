use ducc::Ducc;
use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

const BROWSER_ENV_SHIM: &str = include_str!("./browser_env_shim.js");

pub struct JsEngine {
    vm: Ducc,
}

impl JsEngine {
    pub fn new() -> Result<Self, ducc::Error> {
        let vm = Ducc::new();
        vm.exec(BROWSER_ENV_SHIM, None, Default::default())?;

        Ok(JsEngine { vm })
    }

    pub fn exec(&self, data: &str) -> Result<(), ducc::Error> {
        self.vm.exec(data, Some("main"), Default::default())?;
        Ok(())
    }

    pub fn get_global<'a, K: ducc::ToValue<'a>, V: ducc::FromValue<'a>>(
        &'a self,
        name: K,
    ) -> Result<V, ducc::Error> {
        let globals = self.vm.globals();
        let prop = globals.get(name)?;
        V::from_value(prop, &self.vm)
    }

    pub fn get_ducc(&self) -> &Ducc {
        &self.vm
    }

    pub fn into_ducc(self) -> Ducc {
        self.vm
    }
}

pub fn get_time_ms() -> u128 {
    let start = SystemTime::now();
    start.duration_since(UNIX_EPOCH).unwrap().as_millis()
}
