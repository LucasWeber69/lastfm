use crate::{
    errors::AppError,
    middleware::AuthUser,
    models::{UpdateUser, User},
    AppState,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};

pub async fn get_me(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&auth_user.user_id)
        .fetch_optional(&app_state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn update_me(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(update_user): Json<UpdateUser>,
) -> Result<Json<User>, AppError> {
    // Build update query safely using SQLx query builder pattern
    // This prevents SQL injection by using parameterized queries
    let mut query_parts = Vec::new();
    let mut has_updates = false;

    if update_user.name.is_some() {
        query_parts.push("name");
        has_updates = true;
    }
    if update_user.bio.is_some() {
        query_parts.push("bio");
        has_updates = true;
    }
    if update_user.birth_date.is_some() {
        query_parts.push("birth_date");
        has_updates = true;
    }
    if update_user.gender.is_some() {
        query_parts.push("gender");
        has_updates = true;
    }
    if update_user.looking_for.is_some() {
        query_parts.push("looking_for");
        has_updates = true;
    }
    if update_user.latitude.is_some() {
        query_parts.push("latitude");
        has_updates = true;
    }
    if update_user.longitude.is_some() {
        query_parts.push("longitude");
        has_updates = true;
    }

    if !has_updates {
        return Err(AppError::Validation("No fields to update".to_string()));
    }

    // Use individual update queries to avoid SQL injection
    // This is safer than dynamic query building
    if let Some(name) = update_user.name {
        sqlx::query("UPDATE users SET name = ? WHERE id = ?")
            .bind(&name)
            .bind(&auth_user.user_id)
            .execute(&app_state.pool)
            .await?;
    }
    if let Some(bio) = update_user.bio {
        sqlx::query("UPDATE users SET bio = ? WHERE id = ?")
            .bind(&bio)
            .bind(&auth_user.user_id)
            .execute(&app_state.pool)
            .await?;
    }
    if let Some(birth_date) = update_user.birth_date {
        sqlx::query("UPDATE users SET birth_date = ? WHERE id = ?")
            .bind(birth_date)
            .bind(&auth_user.user_id)
            .execute(&app_state.pool)
            .await?;
    }
    if let Some(gender) = update_user.gender {
        sqlx::query("UPDATE users SET gender = ? WHERE id = ?")
            .bind(&gender)
            .bind(&auth_user.user_id)
            .execute(&app_state.pool)
            .await?;
    }
    if let Some(looking_for) = update_user.looking_for {
        sqlx::query("UPDATE users SET looking_for = ? WHERE id = ?")
            .bind(&looking_for)
            .bind(&auth_user.user_id)
            .execute(&app_state.pool)
            .await?;
    }
    if let Some(latitude) = update_user.latitude {
        sqlx::query("UPDATE users SET latitude = ? WHERE id = ?")
            .bind(latitude)
            .bind(&auth_user.user_id)
            .execute(&app_state.pool)
            .await?;
    }
    if let Some(longitude) = update_user.longitude {
        sqlx::query("UPDATE users SET longitude = ? WHERE id = ?")
            .bind(longitude)
            .bind(&auth_user.user_id)
            .execute(&app_state.pool)
            .await?;
    }

    // Fetch updated user
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&auth_user.user_id)
        .fetch_optional(&app_state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}

pub async fn get_user(
    Path(user_id): Path<String>,
    State(app_state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&app_state.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(Json(user))
}
