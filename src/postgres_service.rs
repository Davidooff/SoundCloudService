use sqlx::postgres::{PgHasArrayType, PgPoolOptions, PgTypeInfo};
use sqlx::{pool, PgPool, Postgres};
use std::error::Error;
// Types:
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "author_input")] // Links this struct to the PG type
pub struct AuthorInput {
    id: i32,
    title: String,
    img: String,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "album_input")]
pub struct AlbumInput {
    id: i32,
    title: String,
    img: String,
    author_id: i32,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "track_input")]
pub struct TrackInput {
    id: i32,
    title: String,
    duration: i32,
}

// Tables:
#[derive(Debug, sqlx::FromRow)]
pub struct TrackTblEntry {
    id: i32,
    title: String,
    duration: i32,
    img: Option<String>,
    author_id: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct OptionalStr (Option<String>);

impl PgHasArrayType for TrackInput {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_tracks_input")
    }
}

pub struct PostgresDb {
    pool: pool::Pool<Postgres>
}

impl PostgresDb {
    pub async fn new(url: String) -> Self {
        Self {
            pool: PgPoolOptions::new().connect(&url).await.unwrap()
        }
    }

    pub async fn create_album
        (&self, tracks: &[TrackInput], album: &AlbumInput, author: &AuthorInput)
            -> Result<(), Box<dyn Error>> {
        
        sqlx::query("CALL add_album($1, $2, $3)")
            .bind(tracks)
            .bind(album)
            .bind(author)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    
    pub async fn add_track(&self, track: &TrackInput, track_img: String, author: &AuthorInput) 
            -> Result<(), Box<dyn Error>> {
        
        sqlx::query("CALL add_track($1, $2, $3)")
            .bind(track)
            .bind(&track_img)
            .bind(author)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_tracks(&self, id: &str) -> Result<TrackTblEntry, Box<dyn Error>> {
        let mut track: TrackTblEntry = sqlx::query_as("SELECT * FROM tracks WHERE id=$1")
            .bind(id)
            .fetch_one(&self.pool).await?;
        if track.img == None{
            let img:OptionalStr = sqlx::query_as("SELECT img FROM albums WHERE id=$1").bind(id).fetch_one(&self.pool).await?;
            track.img = img.0;
        }

        Ok(track)
    }   
}   