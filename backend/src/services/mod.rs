pub mod auth_service;
pub mod lastfm_service;
pub mod compatibility_service;
pub mod photo_service;
pub mod match_service;

pub use auth_service::AuthService;
pub use lastfm_service::LastFmService;
pub use compatibility_service::CompatibilityService;
pub use photo_service::PhotoService;
pub use match_service::MatchService;
