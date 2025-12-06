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
    // Build dynamic update query
    let mut query = String::from("UPDATE users SET ");
    let mut updates = Vec::new();
    let mut bind_values: Vec<String> = Vec::new();

    if let Some(name) = &update_user.name {
        updates.push("name = ?");
        bind_values.push(name.clone());
    }
    if let Some(bio) = &update_user.bio {
        updates.push("bio = ?");
        bind_values.push(bio.clone());
    }
    if let Some(birth_date) = update_user.birth_date {
        updates.push("birth_date = ?");
        bind_values.push(birth_date.to_string());
    }
    if let Some(gender) = &update_user.gender {
        updates.push("gender = ?");
        bind_values.push(gender.clone());
    }
    if let Some(looking_for) = &update_user.looking_for {
        updates.push("looking_for = ?");
        bind_values.push(looking_for.clone());
    }
    if let Some(latitude) = update_user.latitude {
        updates.push("latitude = ?");
        bind_values.push(latitude.to_string());
    }
    if let Some(longitude) = update_user.longitude {
        updates.push("longitude = ?");
        bind_values.push(longitude.to_string());
    }

    if updates.is_empty() {
        return Err(AppError::Validation("No fields to update".to_string()));
    }

    query.push_str(&updates.join(", "));
    query.push_str(" WHERE id = ?");

    // Execute update (simplified version - in production use a proper query builder)
    sqlx::query(&query)
        .bind(&auth_user.user_id)
        .execute(&app_state.pool)
        .await?;

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
