use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use lastfm_dating_backend::{
    config::Config,
    db,
    middleware::auth_middleware,
    routes,
    services::{AuthService, CompatibilityService, LastFmService, MatchService, PhotoService},
    AppState,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use axum::http::{HeaderValue, Method};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    let host = config.host.clone();
    let port = config.port;
    let allowed_origins = config.allowed_origins.clone();

    // Create database pool
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Database connection established");

    // Initialize services
    let auth_service = Arc::new(AuthService::new(config.clone()));
    let lastfm_service = Arc::new(LastFmService::new(config.clone()));
    let compatibility_service = Arc::new(CompatibilityService::new(lastfm_service.clone()));
    let match_service = Arc::new(MatchService::new(compatibility_service.clone()));
    let photo_service = Arc::new(PhotoService::new(config.clone()));

    let config_arc = Arc::new(config);

    // Create shared app state
    let app_state = AppState {
        pool,
        config: config_arc.clone(),
        auth_service,
        lastfm_service,
        compatibility_service,
        match_service,
        photo_service,
    };

    // Build application routes
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/users/me", get(routes::users::get_me))
        .route("/users/me", put(routes::users::update_me))
        .route("/users/:id", get(routes::users::get_user))
        .route("/likes", post(routes::matches::create_like))
        .route("/matches", get(routes::matches::get_matches))
        .route("/matches/:id", delete(routes::matches::delete_match))
        .route("/photos", post(routes::photos::create_photo))
        .route("/photos/:user_id", get(routes::photos::get_user_photos))
        .route("/photos/:id", delete(routes::photos::delete_photo))
        .route("/lastfm/connect", post(routes::lastfm::connect_lastfm))
        .route("/lastfm/sync", post(routes::lastfm::sync_scrobbles))
        .route("/discover", get(routes::discover::get_discover_profiles))
        .layer(middleware::from_fn_with_state(
            config_arc.clone(),
            auth_middleware,
        ));

    // Combine routes and add CORS with security restrictions
    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origins
                .iter()
                .map(|origin| {
                    origin.parse::<HeaderValue>()
                        .expect(&format!("Invalid ALLOWED_ORIGINS value: {}", origin))
                })
                .collect::<Vec<_>>(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any)
        .allow_credentials(true);

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server listening on {}:{}", host, port);

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
