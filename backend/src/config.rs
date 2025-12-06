use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub lastfm_api_key: String,
    pub lastfm_api_secret: String,
    pub s3_bucket: String,
    pub s3_region: String,
    pub s3_access_key: String,
    pub s3_secret_key: String,
    pub host: String,
    pub port: u16,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        let allowed_origins = env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            jwt_secret: env::var("JWT_SECRET")?,
            lastfm_api_key: env::var("LASTFM_API_KEY")?,
            lastfm_api_secret: env::var("LASTFM_API_SECRET")?,
            s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "lastfm-dating-photos".to_string()),
            s3_region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            s3_access_key: env::var("S3_ACCESS_KEY").unwrap_or_default(),
            s3_secret_key: env::var("S3_SECRET_KEY").unwrap_or_default(),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            allowed_origins,
        })
    }
}
