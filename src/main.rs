mod soundcloud_api;
mod own_api_domain;
mod routs;
mod postgres_service;

use std::sync::Arc;
use axum::Router;
use axum::routing::get;
use dotenvy::dotenv;
use envy;

use crate::routs::{get_stream, get_tracks_data};
use crate::soundcloud_api::{SoundCloudApi, TrackData};

struct SharedState {
    soundcloud_api: Arc<SoundCloudApi>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let shared_state = Arc::new(SharedState{ soundcloud_api: Arc::new(SoundCloudApi::new("bARmVKz9fbjpOI0NItFozlgs3kKCmUlT")) });

    let app = Router::new()
        .route("/track_data/{ids}", get(get_tracks_data))
        .route("/chunks/{id}", get(get_stream))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
