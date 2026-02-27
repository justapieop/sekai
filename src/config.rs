use std::env;

pub struct Config {
    pub host: String,
    pub machine_id: u32,
    pub jwks_iss: String,
    pub jwks_url: String,
}

const DEFAULT_HOST: &str = "127.0.0.1:3000";

impl Config {
    pub fn new() -> Self {
        Self {
            host: env::var("HOST").unwrap_or(String::from(DEFAULT_HOST)),
            machine_id: env::var("MACHINE_ID")
                .expect("MACHINE_ID must be set")
                .parse()
                .expect("MACHINE_ID must be a 32 bit unsigned integer"),
            jwks_iss: env::var("JWKS_ISS").expect("JWKS_ISS must be set"),
            jwks_url: env::var("JWKS_URL").expect("JWKS_URL must be set"),
        }
    }
}
