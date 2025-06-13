use std::convert::Infallible;
use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, Response, StatusCode};
use axum::Json;
use axum::response::IntoResponse;
use futures::{stream, Stream, TryFutureExt};
use futures::StreamExt;
use std::error::Error;
use std::pin::Pin;
use serde::Deserialize;
use crate::{soundcloud_api, SharedState};
use crate::postgres_service::{AuthorInput, TrackInput, TrackTblEntry};
use crate::soundcloud_api::{ByteStream, SoundCloudApi};

#[derive(Deserialize, Debug)]
pub struct SearchParams {
    q: String,
    limit: String,
    offset: String,
}

#[axum::debug_handler]
pub async fn search(Query(params): Query<SearchParams>, State(state): State<Arc<SharedState>>) -> impl IntoResponse {
    let soundcloud = state.soundcloud_api.clone();

    let seach_res = soundcloud.search(&params.q, &params.offset, &params.limit).await;
}

#[axum::debug_handler]
pub async fn get_tracks_data(Path(ids): Path<String>, State(state): State<Arc<SharedState>>) -> Result<impl IntoResponse, StatusCode> {
    let soundcloud = state.soundcloud_api.clone();
    let postgre = state.postgres_db.clone();

    // The original `get_track_data` is fine
    let tracks_data = soundcloud.get_track_data(ids.as_str()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    for track in &tracks_data {
        let ti = TrackInput::from(track);
        let ai = AuthorInput::from(track);

        // This call remains the same
        postgre.add_track(&ti, &track.artwork_url, &ai).await.expect("failed to add track");
    }

    Ok(Json(tracks_data))
}


#[axum::debug_handler]
pub async fn get_stream(Path(id): Path<String>, State(state): State<Arc<SharedState>>) -> Result<Response<Body>, StatusCode> {
        let soundcloud = state.soundcloud_api.clone();

        let track_data = soundcloud.get_track_data(id.as_str()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let track = track_data.first().ok_or(StatusCode::BAD_REQUEST)?;
        let media_data = track.media.transcodings.first().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let url_with_chunks = soundcloud.get_url_to_chunks(media_data.url.as_str(), track.track_authorization.as_str())
            .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let chunks = soundcloud.get_chunks(url_with_chunks.as_str()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        println!("{:?}", chunks);

        let chunk_futures = stream::iter(chunks.into_iter())
            .map(move |chunk_token| {
                let sc = soundcloud.clone();

                async move {
                    sc.stream_chunk(chunk_token).await
                }
            });


        let max_concurrency = 1;
        let combined_stream =
            chunk_futures
                // drive up to `max_concurrency` in parallel:
                .buffered(max_concurrency)
                // . Now each item is a ByteStream:
                .flatten()
                // box it so it fits the `Body::from_stream` signature:
                .boxed();

        let body = Body::from_stream(combined_stream);

        let response = Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                "application/octet-stream",
            )
            // .header(
            //     header::CONTENT_DISPOSITION,
            //     "attachment; filename=\"combined_data.bin\"",
            // )
            .body(body)
            // If building the response fails (e.g., invalid header), return an error.
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        println!("Handler finished: Streaming response to client.");
        Ok(response)
}