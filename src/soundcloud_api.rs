use std::error::Error;
use std::pin::Pin;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use regex::Regex;
use futures::{Stream, StreamExt};

const BASE_URL: &str = "https://api-v2.soundcloud.com";

#[derive(Deserialize, Serialize)]
pub struct FormatData {
    pub protocol: String,
    pub mime_type: String
}

#[derive(Deserialize, Serialize)]
pub struct EncodingData {
    pub url: String,
    pub preset: String,
    pub duration: u32,
    pub snipped: bool,
    pub format: FormatData,
    pub quality: String,
    pub is_legacy_transcoding: bool,
}

#[derive(Deserialize, Serialize)]
pub struct Media {
    pub transcodings: Vec<EncodingData>,
}

#[derive(Deserialize, Serialize)]
pub struct User {
    pub avatar_url: String,
    pub username: String,
    pub id: u32,
}

#[derive(Deserialize, Serialize)]
pub struct TrackData {
    pub id: u32,
    pub title: String,
    pub artwork_url:  String,
    pub duration: u32,
    pub media: Media,
    pub track_authorization: String,
    pub policy: String,
    pub user: User
}


#[derive(Deserialize, Serialize)]
pub struct ChunkUrl {
    pub url: String,
}

pub struct SoundCloudApi {
    client: Client,
    client_id: String,
    url_re: Regex,
}

pub type ByteStream = Pin<Box<dyn Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Send>>;

impl SoundCloudApi {
    pub fn new(client_id: &str) -> Self {
        Self {
            client: Client::new(),
            client_id: String::from(client_id),
            url_re: Regex::new(r#"https:?:[^\s"]+"#).unwrap()
        }
    }

    pub async fn get_track_data(&self, ids: &str) -> Result<Vec<TrackData>, Box<dyn Error>> {
        let url = Url::parse_with_params(format!("{}/tracks", BASE_URL).as_str(), &[
            ("ids", ids), ("client_id", self.client_id.as_str()),
        ])?;
        let req = self.client.get(url).build()?;
        let res = self.client.execute(req).await?.text().await?;
        
        let track: Vec<TrackData> = serde_json::from_str(&res)?;

        Ok(track)
    }

    pub async fn get_url_to_chunks(&self, url: &str, track_authorization: &str) -> Result<String, Box<dyn Error>> {
        let url = Url::parse_with_params(url, &[
            ("client_id", self.client_id.as_str()), ("track_authorization", track_authorization),
        ])?;
        let req = self.client.get(url).build()?;
        let res = self.client.execute(req).await?.text().await?;
        let urls: ChunkUrl = serde_json::from_str(&res)?;

        Ok(urls.url)
    }

    pub async fn get_chunks(&self, url: &str) -> Result<Vec<String>, Box<dyn Error>> {
        let req = self.client.get(url).build()?;
        let res = self.client.execute(req).await?.text().await?;
        let urls: Vec<String> = self.url_re.find_iter(res.as_str()).map(|m| m.as_str().to_string()).collect();
        
        Ok(urls)
    }

    pub async fn stream_chunk(&self, url: String) -> Result<ByteStream, Box<dyn Error>> {
        let response = self.client
            .get(url)
            .send()
            .await?;

        Ok(response.bytes_stream().boxed())
    }
}