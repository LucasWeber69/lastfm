use crate::{
    config::Config,
    db::DbPool,
    errors::AppError,
    models::{CreatePhoto, Photo},
};
use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region},
    Client as S3Client,
};
use uuid::Uuid;

pub struct PhotoService {
    config: Config,
    s3_client: Option<S3Client>,
}

impl PhotoService {
    pub fn new(config: Config) -> Self {
        Self { 
            config,
            s3_client: None,
        }
    }

    pub async fn with_s3(mut self) -> Self {
        self.s3_client = Self::create_s3_client(&self.config).await.ok();
        self
    }

    async fn create_s3_client(config: &Config) -> Result<S3Client, AppError> {
        let credentials = Credentials::new(
            &config.s3_access_key,
            &config.s3_secret_key,
            None,
            None,
            "static",
        );

        let s3_config = aws_sdk_s3::config::Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(Region::new(config.s3_region.clone()))
            .endpoint_url(&config.s3_endpoint)
            .force_path_style(true) // Required for MinIO
            .build();

        Ok(S3Client::from_conf(s3_config))
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

    // Upload a photo to S3/MinIO and return the URL
    pub async fn upload_photo(&self, file_data: Vec<u8>, original_filename: &str, content_type: &str) -> Result<String, AppError> {
        // Validate file size (max 5MB)
        if file_data.len() > 5 * 1024 * 1024 {
            return Err(AppError::Validation("File size exceeds 5MB limit".to_string()));
        }

        // Validate content type
        let valid_types = ["image/jpeg", "image/png", "image/webp", "image/gif"];
        if !valid_types.contains(&content_type) {
            return Err(AppError::Validation(
                "Invalid file type. Only JPEG, PNG, WebP, and GIF are allowed".to_string()
            ));
        }

        // Generate unique filename (we use a UUID instead of original filename for security)
        let photo_id = Uuid::new_v4();
        let extension = match content_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "jpg",
        };
        let key = format!("photos/{}.{}", photo_id, extension);
        
        tracing::info!("Uploading photo '{}' as '{}'", original_filename, key);

        // Upload to S3/MinIO if client is available
        if let Some(client) = &self.s3_client {
            let put_object = client
                .put_object()
                .bucket(&self.config.s3_bucket)
                .key(&key)
                .body(file_data.into())
                .content_type(content_type)
                .send()
                .await;

            match put_object {
                Ok(_) => {
                    // Return the URL
                    let url = if self.config.s3_endpoint.contains("localhost") || self.config.s3_endpoint.contains("127.0.0.1") {
                        // For local MinIO, construct URL
                        format!("{}/{}/{}", self.config.s3_endpoint, self.config.s3_bucket, key)
                    } else {
                        // For production S3, use standard URL format
                        format!("https://{}.s3.{}.amazonaws.com/{}", 
                            self.config.s3_bucket, self.config.s3_region, key)
                    };
                    Ok(url)
                }
                Err(e) => {
                    tracing::error!("Failed to upload to S3/MinIO: {}", e);
                    Err(AppError::Internal(format!("Failed to upload photo: {}", e)))
                }
            }
        } else {
            // Fallback: Return placeholder URL if S3 client not configured
            tracing::warn!("S3 client not configured, using placeholder URL");
            Ok(format!("https://placeholder.com/photos/{}.{}", photo_id, extension))
        }
    }
}
