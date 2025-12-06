use crate::{
    config::Config,
    db::DbPool,
    services::{AuthService, CompatibilityService, LastFmService, MatchService, PhotoService},
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
}
