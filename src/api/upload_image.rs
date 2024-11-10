use super::models::ImageData;
use crate::{image_processing::DEFAULT_IMAGE_SIZES, storage::s3_client::S3Uploader};
use actix_web::{web, HttpResponse, Responder, http::header::ContentType};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use tokio::task;
use uuid::Uuid;
use image::codecs::jpeg::JpegEncoder;
use tokio::spawn;

async fn resize_and_upload_image(
    image_data: Vec<u8>,
    main_image_id: Uuid,
    s3_uploader: web::Data<S3Uploader>,
    pool: &PgPool,  // Added missing pool parameter
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::load_from_memory(&image_data)?;
    let mut handles = vec![];
    let mut size_urls = Vec::new();  


    for &(width, height) in DEFAULT_IMAGE_SIZES.iter() {
        let resized_img = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        let mut buffer = vec![];
        let mut cursor = std::io::Cursor::new(&mut buffer);
        JpegEncoder::new_with_quality(&mut cursor, 80).encode_image(&resized_img)?;
        
        let resized_id = Uuid::new_v4();
        let resized_file_name = format!("images/{}/{}.jpg", main_image_id, resized_id);
        
        let s3_uploader = s3_uploader.clone();
        let buffer_clone = buffer.clone();
        let size = (width, height);

        handles.push(task::spawn(async move {
            match s3_uploader.upload_image(buffer_clone, resized_file_name).await {
                Ok(url) => Ok((size, url)),
                Err(e) => Err(e)
            }
        }));
    }

    
    for handle in handles {
        if let Ok(Ok((size, url))) = handle.await {
            size_urls.push((size, url));
        }
    }

    
    if !size_urls.is_empty() {
        ImageData::update_resized_urls(pool, main_image_id, size_urls).await?;
    }

    Ok(())
}

async fn upload_image(
    db_pool: web::Data<PgPool>,
    s3_uploader: web::Data<S3Uploader>,
    image_data: web::Bytes,
) -> impl Responder {
    let image_id = Uuid::new_v4();
    let created_at = Utc::now();
    let file_name = format!("images/{}/original.jpg", image_id);
    
    // Read payload code here...
    
    match s3_uploader.upload_image(image_data.to_vec(), file_name.clone()).await {
        Ok(url) => {
            let image = ImageData {
                id: image_id,
                url: url.clone(),
                created_at: Some(created_at),
                resized_urls: Some(json!({})),
            };
            
            let s3_uploader_clone = s3_uploader.clone();
            let image_data_clone = image_data.to_vec();
            let db_pool_clone = db_pool.clone();
            
            spawn(async move {
                if let Err(e) = resize_and_upload_image(
                    image_data_clone,
                    image_id,
                    web::Data::new((*s3_uploader_clone).as_ref().clone()),
                    &db_pool_clone,  
                ).await {
                    eprintln!("Failed to resize image: {:?}", e);
                }
            });

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
            .app_data(web::PayloadConfig::new(10 * 1024 * 1024))
            .route(web::post().to(upload_image))
    );
}