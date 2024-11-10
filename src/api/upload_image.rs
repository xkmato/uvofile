use super::models::ImageData;
use crate::storage::s3_client::S3Uploader;
use actix_web::{web, HttpResponse, Responder, http::header::ContentType};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn upload_image(
    db_pool: web::Data<PgPool>,
    s3_uploader: web::Data<S3Uploader>,
    image_data: web::Bytes,
) -> impl Responder {
    let image_id = Uuid::new_v4();
    let mime_type = match infer::get(&image_data) {
        Some(mime) => mime,
        None => {
            eprintln!("Failed to infer mime type");
            return HttpResponse::BadRequest()
                .insert_header(ContentType::json())
                .json(json!({"error": "Invalid image data"}));
        }
    };
    
    let extension = match mime_type.extension() {
        "jpeg" | "jpg" => "jpg",
        "png" => "png",
        "gif" => "gif",
        _ => {
            eprintln!("Unsupported image type: {}", mime_type.extension());
            return HttpResponse::BadRequest()
                .insert_header(ContentType::json())
                .json(json!({"error": "Unsupported image type"}));
        }
    };

    let file_name = format!("images/{}.{}", image_id, extension);
    let created_at = Utc::now();

    match s3_uploader.upload_image(image_data.to_vec(), file_name.clone()).await {
        Ok(url) => {
            let image = ImageData {
                id: image_id,
                url: url.clone(),
                created_at: Some(created_at),
            };
            match image.insert(&db_pool).await {
                Ok(_) => HttpResponse::Ok()
                    .insert_header(ContentType::json())
                    .json(json!({"url": url})),
                Err(e) => {
                    eprintln!("Failed to insert image record: {:?}", e);
                    HttpResponse::InternalServerError()
                        .insert_header(ContentType::json())
                        .json(json!({"error": "Failed to insert image record"}))
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to upload image to S3: {:?}", e);
            HttpResponse::InternalServerError()
                .insert_header(ContentType::json())
                .json(json!({"error": "Failed to upload image to S3"}))
        }
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/upload")
            .app_data(web::PayloadConfig::new(10 * 1024 * 1024)) // Set payload limit to 10 MB
            .route(web::post().to(upload_image))
    );
}