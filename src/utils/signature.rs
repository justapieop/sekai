use hmac::{Hmac, Mac};
use sha2::Sha256;

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

    pub fn verify(&mut self, msg: &[u8], received_hmac: &[u8]) -> bool {
        self.hmac.update(msg);
        let res = self.hmac.clone().finalize();

        res.into_bytes().as_slice() == received_hmac
    }
}
