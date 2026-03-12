use std::env;

pub struct Config {
    pub host: String,
    pub machine_id: u32,
    pub jwks_iss: String,
    pub jwks_url: String,
    pub database_url: String,
    pub s3_endpoint: String,
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_region: String,
    pub authgear_webhook_secret: String,
}

const DEFAULT_HOST: &str = "127.0.0.1:3000";
const DEFAULT_S3_REGION: &str = "us-east-1";

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
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            s3_endpoint: env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set"),
            s3_access_key_id: env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID must be set"),
            s3_secret_access_key: env::var("S3_SECRET_ACCESS_KEY")
                .expect("S3_SECRET_ACCESS_KEY must be set"),
            s3_region: env::var("S3_REGION").unwrap_or(String::from(DEFAULT_S3_REGION)),
            authgear_webhook_secret: env::var("AUTHGEAR_WEBHOOK_SECRETS")
                .expect("AUTHGEAR_WEBHOOK_SECRETS must be set"),
        }
    }
}
