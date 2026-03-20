use bytes::Bytes;
use moka::future::Cache;
use s3::{
    Client, Credentials, Error,
    types::{GetObjectOutput, PutObjectOutput},
};
use std::time::Duration;
use uuid::Uuid;

const PUBLIC_DIR_NAME: &str = "public";

pub struct StorageUtils {
    s3_client: Client,
    cache: Cache<String, Bytes>,
    bucket_name: String,
}

impl StorageUtils {
    pub fn new(
        endpoint: &str,
        region: &str,
        access_key_id: &str,
        secret_access_key: &str,
        bucket_name: &str,
    ) -> Self {
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
            cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_hours(1))
                .build(),
            bucket_name: String::from(bucket_name),
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
                self.bucket_name.clone(),
                format!("{}/{}", user_id.to_string(), file_name),
            )
            .content_type(content_type)
            .body_bytes(data)
            .send()
            .await
    }

    pub async fn delete_public_file(&self, file_name: &str) -> Result<(), Error> {
        self.cache.invalidate_all();
        match self
            .s3_client
            .objects()
            .delete(
                self.bucket_name.clone(),
                format!("{}/{}", PUBLIC_DIR_NAME, file_name),
            )
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn upload_public_file(
        &self,
        data: &Bytes,
        file_name: &str,
        content_type: &str,
    ) -> Result<PutObjectOutput, Error> {
        self.cache
            .insert(String::from(file_name), data.clone())
            .await;
        self.s3_client
            .objects()
            .put(
                self.bucket_name.clone(),
                format!("{}/{}", PUBLIC_DIR_NAME, file_name),
            )
            .content_type(content_type)
            .body_bytes(data.clone())
            .send()
            .await
    }

    pub async fn fetch_public_file(&self, file_name: &str) -> Result<Bytes, Error> {
        if let Some(s) = self.cache.get(file_name).await {
            return Ok(s);
        }
        match self
            .s3_client
            .objects()
            .get(
                self.bucket_name.clone(),
                format!("{}/{}", PUBLIC_DIR_NAME, file_name),
            )
            .send()
            .await
        {
            Ok(s) => match s.bytes().await {
                Ok(b) => Ok(b),
                Err(e) => Err(e.into()),
            },
            Err(e) => Err(e.into()),
        }
    }

    pub async fn fetch_file(
        &self,
        user_id: Uuid,
        file_name: &str,
    ) -> Result<GetObjectOutput, Error> {
        self.s3_client
            .objects()
            .get(
                self.bucket_name.clone(),
                format!("{}/{}", user_id.to_string(), file_name),
            )
            .send()
            .await
    }

    pub async fn delete_file(&self, user_id: Uuid, file_name: &str) -> Result<(), Error> {
        self.s3_client
            .objects()
            .delete(
                self.bucket_name.clone(),
                format!("{}/{}", user_id.to_string(), file_name),
            )
            .send()
            .await
    }
}
