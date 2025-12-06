use crate::{
    config::Config,
    db::DbPool,
    errors::AppError,
    models::{Artist, Scrobble},
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct LastFmTopArtistsResponse {
    topartists: TopArtists,
}

#[derive(Debug, Deserialize)]
struct TopArtists {
    artist: Vec<LastFmArtist>,
}

#[derive(Debug, Deserialize)]
struct LastFmArtist {
    name: String,
    mbid: Option<String>,
    playcount: String,
    listeners: String,
}

pub struct LastFmService {
    config: Config,
    client: Client,
}

impl LastFmService {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub async fn sync_user_scrobbles(
        &self,
        pool: &DbPool,
        user_id: &str,
        lastfm_username: &str,
    ) -> Result<Vec<Artist>, AppError> {
        // Fetch top artists from Last.fm
        let artists = self.fetch_top_artists(lastfm_username, "6month", 50).await?;

        // Clear old cached data for this user and period
        sqlx::query("DELETE FROM scrobbles_cache WHERE user_id = ? AND period = ?")
            .bind(user_id)
            .bind("6month")
            .execute(pool)
            .await?;

        // Insert new data
        for artist in &artists {
            let scrobble = Scrobble::new(
                user_id.to_string(),
                artist.name.clone(),
                artist.mbid.clone(),
                artist.play_count,
                artist.listeners,
                "6month".to_string(),
            );

            sqlx::query(
                "INSERT INTO scrobbles_cache (id, user_id, artist_name, artist_mbid, play_count, listeners, period) VALUES (?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(&scrobble.id)
            .bind(&scrobble.user_id)
            .bind(&scrobble.artist_name)
            .bind(&scrobble.artist_mbid)
            .bind(scrobble.play_count)
            .bind(scrobble.listeners)
            .bind(&scrobble.period)
            .execute(pool)
            .await?;
        }

        Ok(artists)
    }

    async fn fetch_top_artists(
        &self,
        username: &str,
        period: &str,
        limit: u32,
    ) -> Result<Vec<Artist>, AppError> {
        let url = format!(
            "https://ws.audioscrobbler.com/2.0/?method=user.gettopartists&user={}&period={}&limit={}&api_key={}&format=json",
            username, period, limit, self.config.lastfm_api_key
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to fetch Last.fm data: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ExternalApi(format!(
                "Last.fm API returned status: {}",
                response.status()
            )));
        }

        let data: LastFmTopArtistsResponse = response
            .json()
            .await
            .map_err(|e| AppError::ExternalApi(format!("Failed to parse Last.fm response: {}", e)))?;

        Ok(data
            .topartists
            .artist
            .into_iter()
            .map(|a| Artist {
                name: a.name,
                mbid: if a.mbid.as_ref().map_or(true, |s| s.is_empty()) {
                    None
                } else {
                    a.mbid
                },
                play_count: a.playcount.parse().unwrap_or(0),
                listeners: a.listeners.parse().unwrap_or(0),
            })
            .collect())
    }

    pub async fn get_user_top_artists(
        &self,
        pool: &DbPool,
        user_id: &str,
        limit: i32,
    ) -> Result<Vec<Artist>, AppError> {
        let scrobbles = sqlx::query_as::<_, Scrobble>(
            "SELECT * FROM scrobbles_cache WHERE user_id = ? AND period = '6month' ORDER BY play_count DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(pool)
        .await?;

        Ok(scrobbles
            .into_iter()
            .map(|s| Artist {
                name: s.artist_name,
                mbid: s.artist_mbid,
                play_count: s.play_count,
                listeners: s.listeners,
            })
            .collect())
    }
}
