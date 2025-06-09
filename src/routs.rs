use std::sync::Arc;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, Response, StatusCode};
use axum::Json;
use axum::response::IntoResponse;
use futures::{TryFutureExt};
use futures::StreamExt;
use std::error::Error;
use std::pin::Pin;
use crate::{SharedState};
use crate::soundcloud_api::{ByteStream, SoundCloudApi};

#[axum::debug_handler]
pub async fn get_tracks_data(Path(ids): Path<String>, State(state): State<Arc<SharedState>>) -> Result<impl IntoResponse, StatusCode> {
    let soundcloud = state.soundcloud_api.clone();
    let tracks_data = soundcloud.get_track_data(ids.as_str()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(tracks_data))
}


#[axum::debug_handler]
pub async fn get_stream(Path(id): Path<String>, State(state): State<Arc<SharedState>>) -> Result<Response<Body>, StatusCode> {
    {
        let soundcloud = state.soundcloud_api.clone();

        let track_data = soundcloud.get_track_data(id.as_str()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let track = track_data.first().ok_or(StatusCode::BAD_REQUEST)?;
        let media_data = track.media.transcodings.first().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

        let url_with_chunks = soundcloud.get_url_to_chunks(media_data.url.as_str(), track.track_authorization.as_str())
            .await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let chunks = soundcloud.get_chunks(url_with_chunks.as_str()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        println!("{:?}", chunks);

        let mut final_stream: ByteStream = Box::pin(futures::stream::empty());
        let mut chunk_query:Vec<Pin<Box<dyn Future<Output=Result<ByteStream, Box<dyn Error>>> + Send>>> = vec![];

        for chunk in chunks {
            chunk_query.push(Box::pin(soundcloud.stream_chunk(chunk)));//.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        };

        for  chunk in chunk_query {
            final_stream = final_stream.chain(chunk.await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?).boxed();
        }

        let body = Body::from_stream(final_stream);

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

        //
        //
        //
        // let stream_of_urls = stream::iter(chunks);
        //
        //
        // let final_stream = stream_of_urls.then(move |chunk_url| {
        //     // Clone soundcloud API handle for the closure
        //     let soundcloud = soundcloud.clone();
        //     async move {
        //         println!("Preparing to stream chunk: {}", chunk_url);
        //         soundcloud.stream_chunk(&chunk_url).await
        //     }
        // })
        //     .flat_map(|result_of_stream| {
        //         // `result_of_stream` is a Result<ByteStream, _>
        //         // We convert it into a stream that can be flattened.
        //         match result_of_stream {
        //             Ok(stream) => stream.map(Ok).left_stream(), // `left_stream` helps unify types
        //             Err(e) => {
        //                 eprintln!("Error getting chunk stream: {:?}", e);
        //                 // Create a stream that yields a single error
        //                 let err_bytes: Result<Bytes, std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::Other, "stream creation failed"));
        //                 stream::once(async { err_bytes }).right_stream()
        //             }
        //         }
        //     });
        //
        // // The body can now be created from the boxed stream
        // let body = Body::from_stream(Box::pin(final_stream));

    }
}