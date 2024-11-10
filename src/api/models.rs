use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageData {
    pub id: Uuid,
    pub url: String,
    pub created_at: Option<DateTime<Utc>>,
    pub resized_urls: Option<serde_json::Value>,
}

impl<'r> FromRow<'r, PgRow> for ImageData {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let url: String = row.try_get("url")?;
        let created_at: DateTime<Utc> = row.try_get("created_at")?;
        let id: Uuid = row.try_get("id")?;
        let resized_urls: Option<serde_json::Value> = row.try_get("resized_urls")?;
        Ok(ImageData {
            id,
            url,
            created_at: Some(created_at),
            resized_urls: resized_urls,
        })
    }
}

impl ImageData {
    pub async fn get(pool: &sqlx::PgPool, id: Uuid) -> Result<Self, sqlx::Error> {
        let image_data =
            sqlx::query_as::<_, ImageData>("SELECT id, url, created_at FROM images WHERE id = $1")
                .bind(id)
                .fetch_one(pool)
                .await?;
        Ok(image_data)
    }

    pub async fn insert(&self, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO images (id, url, created_at) VALUES ($1, $2, $3)")
            .bind(self.id)
            .bind(self.url.clone())
            .bind(self.created_at)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update(pool: &sqlx::PgPool, id: Uuid, url: String) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE images SET url = $1 WHERE id = $2")
            .bind(url)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(pool: &sqlx::PgPool, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM images WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_resized_urls(
        pool: &sqlx::PgPool,
        main_image_id: Uuid,
        size_urls: Vec<((u32, u32), String)>,
    ) -> Result<(), sqlx::Error> {
        let urls_map: HashMap<String, String> = size_urls
                .into_iter()
                .map(|((width, height), url)| {
                    (format!("{}x{}", width, height), url)
                })
                    .collect::<HashMap<String, String>>();
            
        let urls_json = json!(urls_map);
    
        sqlx::query("UPDATE images SET resized_urls = $1 WHERE id = $2")
            .bind(urls_json)
            .bind(main_image_id)
        .execute(pool)
        .await?;
        Ok(())
    }
}
