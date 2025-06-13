mod soundcloud_api;
mod own_api_domain;
mod routs;
mod postgres_service;

use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use dotenvy::dotenv;
use envy;
use crate::postgres_service::PostgresDb;
use crate::routs::{get_stream, get_tracks_data, search};
use crate::soundcloud_api::{SoundCloudApi, TrackData};

struct SharedState {
    soundcloud_api: Arc<SoundCloudApi>,
    postgres_db: Arc<PostgresDb>
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let shared_state = Arc::new(
        SharedState{ 
            soundcloud_api: Arc::new(SoundCloudApi::new("bARmVKz9fbjpOI0NItFozlgs3kKCmUlT")),
            postgres_db: Arc::new(PostgresDb::new("postgres://admin:admin@localhost:5432/musicdb").await)
        });
    

    let app = Router::new()
        .route("/track_data/{ids}", get(get_tracks_data))
        .route("/search", get(search))
        .route("/stream/{id}", get(get_stream))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
