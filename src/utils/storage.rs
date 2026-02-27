use s3::{Client, Credentials};

pub struct StorageUtils {
    s3_client: Client,
}

impl StorageUtils {
    pub fn new(endpoint: &str, access_key_id: &str, secret_access_key: &str) -> Self {
        Self {
            s3_client: Client::builder(endpoint)
                .expect("S3_ENDPOINT must be a valid S3 instance")
                .auth(s3::Auth::Static(
                    Credentials::new(access_key_id, secret_access_key)
                        .expect("S3_ACCESS_KEY_ID AND S3_SECRET_ACCESS_KEY must be valid"),
                ))
                .build()
                .expect("S3_ENDPOINT must be a valid S3 instance"),
        }
    }
}
