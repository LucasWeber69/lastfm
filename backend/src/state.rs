use crate::{
    config::Config,
    db::DbPool,
    services::{
        AuthService, CacheService, CaptchaService, CompatibilityService, LastFmService,
        MatchService, NotificationService, PhotoService, WebSocketService,
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Arc<Config>,
    pub auth_service: Arc<AuthService>,
    pub lastfm_service: Arc<LastFmService>,
    pub compatibility_service: Arc<CompatibilityService>,
    pub match_service: Arc<MatchService>,
    pub photo_service: Arc<PhotoService>,
    pub captcha_service: Arc<CaptchaService>,
    pub cache_service: Arc<CacheService>,
    pub websocket_service: Arc<WebSocketService>,
    pub notification_service: Arc<NotificationService>,
}
