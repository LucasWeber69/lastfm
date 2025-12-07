use crate::{
    config::Config,
    db::DbPool,
    errors::AppError,
    models::{CreatePhoto, Photo},
};
use uuid::Uuid;

pub struct PhotoService {
    config: Config,
}

impl PhotoService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn add_photo(
        &self,
        pool: &DbPool,
        user_id: &str,
        create_photo: CreatePhoto,
    ) -> Result<Photo, AppError> {
        // Check if user already has 6 photos
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM photos WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        if count >= 6 {
            return Err(AppError::Validation("Maximum 6 photos allowed".to_string()));
        }

        let photo = Photo::new(user_id.to_string(), create_photo.url, create_photo.position);

        sqlx::query("INSERT INTO photos (id, user_id, url, position) VALUES (?, ?, ?, ?)")
            .bind(&photo.id)
            .bind(&photo.user_id)
            .bind(&photo.url)
            .bind(photo.position)
            .execute(pool)
            .await?;

        Ok(photo)
    }

    pub async fn get_user_photos(&self, pool: &DbPool, user_id: &str) -> Result<Vec<Photo>, AppError> {
        let photos = sqlx::query_as::<_, Photo>(
            "SELECT * FROM photos WHERE user_id = ? ORDER BY position ASC"
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(photos)
    }

    pub async fn delete_photo(&self, pool: &DbPool, photo_id: &str, user_id: &str) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM photos WHERE id = ? AND user_id = ?")
            .bind(photo_id)
            .bind(user_id)
            .execute(pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Photo not found".to_string()));
        }

        Ok(())
    }

    // NOTE: Development-only placeholder. In production, implement actual S3/MinIO upload
    // This method should upload the file_data to S3/MinIO and return the actual URL
    pub async fn upload_photo(&self, _file_data: Vec<u8>, _filename: &str) -> Result<String, AppError> {
        // TODO: Implement S3/MinIO upload
        // For now, return a placeholder URL for development
        let photo_id = Uuid::new_v4();
        Ok(format!("https://placeholder.com/photos/{}.jpg", photo_id))
    }
}
