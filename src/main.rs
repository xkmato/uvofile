use std::{env, time::Duration};

use actix_web::{web, App, HttpResponse, HttpServer};
use log::info;
use sqlx::postgres::PgPoolOptions;

use dotenv::dotenv;

mod api;
mod background_tasks;
mod config;
mod image_processing;
mod storage;

async fn not_found() -> actix_web::HttpResponse {
    HttpResponse::NotFound().body("Nothing to see here")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    info!("Starting server at http://127.0.0.1:8080");

    let s3_uploader = storage::s3_client::S3Uploader::new()
        .await
        .expect("Failed to create S3 uploader");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .app_data(web::Data::new(s3_uploader.clone()))
            .configure(api::init_routes)
            .default_service(web::route().to(not_found))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
