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
    services::{
        AuthService, CacheService, CaptchaService, CompatibilityService, LastFmService,
        MatchService, NotificationService, PhotoService, WebSocketService,
    },
    AppState,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use axum::http::{HeaderValue, Method};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Helper function for colored output
fn print_success(msg: &str) {
    println!("✓ {}", msg);
}

fn print_error(msg: &str) {
    eprintln!("✗ {}", msg);
}

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
    
    // Initialize cache service
    let cache_service = match CacheService::new(&config.redis_url).await {
        Ok(service) => {
            print_success("Redis connection established");
            Arc::new(service)
        }
        Err(e) => {
            print_error(&format!("Failed to connect to Redis: {}. Application requires Redis.", e));
            tracing::error!("Redis connection failed: {}", e);
            panic!("Redis is required for this application to function properly");
        }
    };
    
    // Initialize photo service with S3
    let photo_service = Arc::new(PhotoService::new(config.clone()).with_s3().await);
    
    // Initialize WebSocket service
    let websocket_service = Arc::new(WebSocketService::new());
    
    // Initialize notification service
    let notification_service = Arc::new(NotificationService::new(
        config.vapid_private_key.clone(),
        config.vapid_public_key.clone(),
        config.vapid_subject.clone(),
    ));
    
    let captcha_service = Arc::new(CaptchaService::new());

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
        captcha_service,
        cache_service,
        websocket_service,
        notification_service,
    };

    // Build application routes
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/auth/register", post(routes::auth::register))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/logout", post(routes::auth::logout))
        .route("/captcha/generate", get(routes::captcha::generate_captcha))
        .route("/captcha/validate", post(routes::captcha::validate_captcha));

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
        // WebSocket route
        .route("/ws", get(routes::websocket::websocket_handler))
        // Notification routes
        .route("/notifications/subscribe", post(routes::notifications::subscribe))
        .route("/notifications/unsubscribe", delete(routes::notifications::unsubscribe))
        .route("/notifications/subscriptions", get(routes::notifications::get_subscriptions))
        // Event routes
        .route("/events/nearby", get(routes::events::get_nearby_events))
        .route("/events/common/:user_id", get(routes::events::get_common_events))
        .route("/events/interests", get(routes::events::get_user_interests))
        .route("/events/interest", post(routes::events::add_interest))
        .route("/events/interest/:event_id", delete(routes::events::remove_interest))
        // Achievement routes
        .route("/achievements", get(routes::achievements::get_achievements))
        .route("/users/me/stats", get(routes::achievements::get_user_stats))
        .layer(middleware::from_fn_with_state(
            config_arc.clone(),
            auth_middleware,
        ));

    // Public routes for achievements (limited view)
    let public_achievement_routes = Router::new()
        .route("/users/:id/achievements", get(routes::achievements::get_user_achievements_public))
        .route("/users/:id/stats", get(routes::achievements::get_user_stats_public))
        .route("/events/popular", get(routes::events::get_popular_events));

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
        .merge(public_achievement_routes)
        .merge(protected_routes)
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", host, port))
        .await
        .expect("Failed to bind to address");

    tracing::info!("Server listening on {}:{}", host, port);

    // Serve with ConnectInfo to access client IP addresses
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server");
}
