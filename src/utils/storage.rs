use bytes::Bytes;
use s3::{
    types::{GetObjectOutput, PutObjectOutput}, Client, Credentials,
    Error,
};
use uuid::Uuid;

const BUCKET_NAME: &str = "assets";
const PUBLIC_BUCKET_NAME: &str = "public";

pub struct StorageUtils {
    s3_client: Client,
}

impl StorageUtils {
    pub fn new(endpoint: &str, region: &str, access_key_id: &str, secret_access_key: &str) -> Self {
        Self {
            s3_client: Client::builder(endpoint)
                .expect("S3_ENDPOINT must be a valid S3 instance")
                .region(region)
                .auth(s3::Auth::Static(
                    Credentials::new(access_key_id, secret_access_key)
                        .expect("S3_ACCESS_KEY_ID AND S3_SECRET_ACCESS_KEY must be valid"),
                ))
                .build()
                .expect("S3_ENDPOINT must be a valid S3 instance"),
        }
    }

    pub async fn upload_file(
        &self,
        user_id: Uuid,
        data: Bytes,
        file_name: &str,
        content_type: &str,
    ) -> Result<PutObjectOutput, Error> {
        self.s3_client
            .objects()
            .put(
                BUCKET_NAME,
                format!("{}/{}", user_id.to_string(), file_name),
            )
            .content_type(content_type)
            .body_bytes(data)
            .send()
            .await
    }

    pub async fn delete_public_file(&self, file_name: &str) -> Result<(), Error> {
        match self
            .s3_client
            .objects()
            .delete(PUBLIC_BUCKET_NAME, file_name)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn upload_public_file(
        &self,
        data: Bytes,
        file_name: &str,
        content_type: &str,
    ) -> Result<PutObjectOutput, Error> {
        self.s3_client
            .objects()
            .put(PUBLIC_BUCKET_NAME, format!("{}", file_name))
            .content_type(content_type)
            .body_bytes(data)
            .send()
            .await
    }

    pub async fn fetch_public_file(&self, file_name: &str) -> Result<GetObjectOutput, Error> {
        self.s3_client
            .objects()
            .get(PUBLIC_BUCKET_NAME, format!("{}", file_name))
            .send()
            .await
    }

    pub async fn fetch_file(
        &self,
        user_id: Uuid,
        file_name: &str,
    ) -> Result<GetObjectOutput, Error> {
        self.s3_client
            .objects()
            .get(
                BUCKET_NAME,
                format!("{}/{}", user_id.to_string(), file_name),
            )
            .send()
            .await
    }

    pub async fn delete_file(&self, user_id: Uuid, file_name: &str) -> Result<(), Error> {
        self.s3_client
            .objects()
            .delete(
                BUCKET_NAME,
                format!("{}/{}", user_id.to_string(), file_name),
            )
            .send()
            .await
    }
}
