use hmac::{Hmac, Mac};
use sha2::Sha256;

#[derive(Debug, Clone)]
pub struct Signature {
    hmac: Hmac<Sha256>,
}

impl Signature {
    pub fn new(key: &str) -> Self {
        Self {
            hmac: Hmac::<Sha256>::new_from_slice(key.as_bytes())
                .expect("AUTHGEAR_WEBHOOK_SECRETS has invalid size"),
        }
    }

    pub fn verify(&self, msg: &[u8], received_hmac: &[u8]) -> bool {
        let mut mac_checker = self.hmac.clone();
        mac_checker.update(msg);
        mac_checker.verify_slice(received_hmac).is_ok()
    }
}
