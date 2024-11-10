pub mod image_data;
pub mod models;
pub mod upload_image;

use actix_web::web;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    image_data::init_routes(cfg);
    upload_image::init_routes(cfg);
}
