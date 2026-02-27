use std::error::Error;

use jsonwebtoken::{Header, Validation, decode_header};
use jwks::Jwks;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
}

#[derive(Clone)]
pub struct JwtUtils {
    jwks_client: Jwks,
    jwks_iss: String,
}

impl JwtUtils {
    pub async fn new(jwks_iss: &str, jwks_url: &str) -> Self {
        Self {
            jwks_client: Jwks::from_jwks_url(String::from(jwks_url))
                .await
                .expect("JWKS_URL must be a valid link pointing to a valid JWKS"),
            jwks_iss: String::from(jwks_iss),
        }
    }

    pub fn verify(&self, jwt: &str) -> Result<String, Box<dyn Error>> {
        let header: Header = match decode_header(jwt) {
            Ok(s) => s,
            Err(e) => {
                return Err(e.into());
            }
        };

        let kid: String = match header.kid {
            Some(s) => s,
            None => {
                return Err("Cannot find kid of the supplied JWT".into());
            }
        };

        let jwk: &jwks::Jwk = match self.jwks_client.keys.get(&kid) {
            Some(v) => v,
            None => return Err("Cannot find a key that matches kid".into()),
        };

        let mut validation = Validation::new(header.alg);
        validation.set_audience(&[self.jwks_iss.clone()]);

        let decode = match jsonwebtoken::decode::<Claims>(&jwt, &jwk.decoding_key, &validation) {
            Ok(v) => v,
            Err(e) => {
                return Err(e.into());
            }
        };

        Ok(decode.claims.sub)
    }
}
