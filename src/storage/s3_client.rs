use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Error};

#[derive(Clone)]
pub struct S3Uploader {
    client: Client,
    bucket: String,
}

impl S3Uploader {
    pub async fn new() -> Result<Self, Error> {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        let bucket = std::env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");

        Ok(S3Uploader { client, bucket })
    }

    pub async fn upload_image(
        &self,
        image_data: Vec<u8>,
        file_name: String,
    ) -> Result<String, Error> {
        let response = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&file_name)
            .body(ByteStream::from(image_data))
            .send()
            .await?;

        let url = response.e_tag().map_or_else(
            || format!("https://{}.s3.amazonaws.com/{}", self.bucket, file_name),
            |_| format!("https://{}.s3.amazonaws.com/{}", self.bucket, file_name),
        );
        Ok(url)
    }
}
