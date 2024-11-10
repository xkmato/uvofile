-- 20241109132000_add_resized_urls_to_images.sql

-- Add resized_urls column
ALTER TABLE images
ADD COLUMN resized_urls JSONB NOT NULL DEFAULT '{}'::jsonb;

-- Rollback
-- DOWN
-- ALTER TABLE images DROP COLUMN resized_urls;