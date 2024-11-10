use super::models::ImageData;
use crate::storage::s3_client::S3Uploader;
use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tokio::task;
use uuid::Uuid;

#[post("/upload")]
async fn upload_image(
    db_pool: web::Data<PgPool>,
    s3_uploader: web::Data<S3Uploader>,
    image_data: web::Bytes,
) -> impl Responder {
    let image_id = Uuid::new_v4();
    let file_name = format!("images/{}.jpg", image_id); // Adjust as needed for image format
    let created_at = Utc::now();

    // Start a background task to upload the image to S3
    let uploader = s3_uploader.clone();
    let image_data_clone = image_data.clone();
    let db_pool_clone = db_pool.clone();
    task::spawn(async move {
        match uploader
            .upload_image(image_data_clone.to_vec(), file_name.clone())
            .await
        {
            Ok(url) => {
                let image = ImageData {
                    id: image_id,
                    url,
                    created_at: Some(created_at),
                };
                // Insert the image record into the database
                if let Err(e) = image.insert(&db_pool_clone).await {
                    eprintln!("Failed to insert image record: {:?}", e);
                }
            }
            Err(e) => eprintln!("Failed to upload image to S3: {:?}", e),
        }
    });

    HttpResponse::Accepted().body("Image upload in progress")
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_image);
}
