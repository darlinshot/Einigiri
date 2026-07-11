use std::env;
use std::collections::HashMap;
use dotenvy;

//#[derive(Debug)]
pub struct AppConfig {
    pub fields: HashMap<String, String>
}

pub fn load_config() -> AppConfig {

    let mut fields = HashMap::new();
    let _ = dotenvy::dotenv().ok();

    if let Ok(envList) = dotenvy::dotenv_iter() {
        for item in envList {
            if let Ok((i, v)) = item {
                fields.insert(i, v);
            }
        }
    }

    AppConfig { fields }
}
