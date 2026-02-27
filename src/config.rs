use std::env;

pub struct Config {
    pub host: String,
}

const DEFAULT_HOST: &str = "";

impl Config {
    pub fn new() -> Self {
        Self {
            host: env::var("HOST").unwrap_or(String::from(DEFAULT_HOST)),
        }
    }
}
