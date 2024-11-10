use super::models::ImageData;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

#[post("/images")]
async fn create_image(db_pool: web::Data<PgPool>, image_url: web::Json<String>) -> impl Responder {
    let new_image: ImageData = ImageData {
        id: Uuid::new_v4(),
        url: image_url.into_inner(),
        created_at: Some(chrono::Utc::now()),
    };

    match new_image.insert(db_pool.get_ref()).await {
        Ok(_) => HttpResponse::Created().json(new_image),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error inserting image: {}", e)),
    }
}

#[get("/images/{id}")]
async fn get_image(db_pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    let id = id.into_inner();
    match ImageData::get(db_pool.get_ref(), id).await {
        Ok(image) => HttpResponse::Ok().json(image),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Image not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[put("/images/{id}")]
async fn update_image(
    db_pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    updated_url: web::Json<String>,
) -> impl Responder {
    let id = id.into_inner();
    let updated_url = updated_url.into_inner();
    let image = match ImageData::get(db_pool.get_ref(), id).await {
        Ok(image) => image,
        Err(sqlx::Error::RowNotFound) => return HttpResponse::NotFound().body("Image not found"),
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("Database error: {}", e))
        }
    };

    match ImageData::update(db_pool.get_ref(), id, updated_url).await {
        Ok(_) => HttpResponse::Ok().json(image),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Image not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

#[delete("/images/{id}")]
async fn delete_image(db_pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    let id = id.into_inner();

    match ImageData::delete(db_pool.get_ref(), id).await {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Image not found"),
        Err(e) => HttpResponse::InternalServerError().body(format!("Database error: {}", e)),
    }
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(create_image);
    cfg.service(get_image);
    cfg.service(update_image);
    cfg.service(delete_image);
}