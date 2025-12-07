use crate::{
    config::Config,
    db::DbPool,
    errors::AppError,
    models::{CreateUser, User},
    services::normalize_email,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // user id
    pub email: String, // user email (for account-based rate limiting)
    pub exp: usize,    // expiration time
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

pub struct AuthService {
    config: Config,
}

impl AuthService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn register(&self, pool: &DbPool, create_user: CreateUser) -> Result<User, AppError> {
        // Normalize email to prevent duplicate account abuse
        let normalized_email = normalize_email(&create_user.email);
        
        // Check if normalized email already exists (prevents gmail+tag abuse)
        let existing = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE LOWER(email) = ? OR email = ?"
        )
            .bind(&normalized_email)
            .bind(&create_user.email)
            .fetch_optional(pool)
            .await?;

        if existing.is_some() {
            return Err(AppError::Validation("Email already registered".to_string()));
        }

        // Hash password
        let password_hash = self.hash_password(&create_user.password)?;

        // Create user with normalized email stored for duplicate detection
        let user = User::new(
            create_user.email,
            password_hash,
            create_user.name,
            create_user.birth_date,
            create_user.gender,
        );

        // Insert into database
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, name, birth_date, gender) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(&user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.name)
        .bind(&user.birth_date)
        .bind(&user.gender)
        .execute(pool)
        .await?;

        Ok(user)
    }

    pub async fn login(&self, pool: &DbPool, login_req: LoginRequest) -> Result<AuthResponse, AppError> {
        // Find user by email
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(&login_req.email)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::Auth("Invalid email or password".to_string()))?;

        // Verify password
        self.verify_password(&login_req.password, &user.password_hash)?;

        // Generate JWT with user ID and email
        let token = self.generate_token(&user.id, &user.email)?;

        Ok(AuthResponse {
            token,
            user: UserResponse {
                id: user.id,
                email: user.email,
                name: user.name,
            },
        })
    }

    pub fn hash_password(&self, password: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))
    }

    pub fn verify_password(&self, password: &str, password_hash: &str) -> Result<(), AppError> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| AppError::Internal(format!("Failed to parse password hash: {}", e)))?;

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|_| AppError::Auth("Invalid email or password".to_string()))
    }

    pub fn generate_token(&self, user_id: &str, email: &str) -> Result<String, AppError> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(30))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: expiration as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("Failed to generate token: {}", e)))
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AppError> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| AppError::Unauthorized)
    }
}
