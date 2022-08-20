//#[macro_use]
//extern crate diesel;
//extern crate dotenv;
pub mod models;
pub mod schema;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::{env, time};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn create_file_row(
    conn: &PgConnection,
    url: String,
    compressed_file_name: String,
) -> models::FileSource {
    use schema::file_source;

    let timestamp = time::SystemTime::now();

    let new_file_source = models::NewFileSource {
        url: url.as_str(),
        compressed_file_name: compressed_file_name.as_str(),
        time_inserted: &timestamp,
    };

    diesel::insert_into(file_source::table)
        .values(&new_file_source)
        .get_result(conn)
        .expect("Error saving new file source")
}

pub fn bulk_insert_music_files(
    conn: &PgConnection,
    new_music_files: Vec<models::NewMusicFiles>,
) -> models::MusicFiles {
    use schema::music_files;

    diesel::insert_into(music_files::table)
        .values(&new_music_files)
        .get_result(conn)
        .expect("Error saving music files")
}
