//#[macro_use]
//extern crate diesel;
//extern crate dotenv;
pub mod schema;
use diesel::types::Timestamp;

pub mod models;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::{env, time};

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
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
        compressed_file_name: &compressed_file_name.as_str(),
        time_inserted: &timestamp,
    };

    diesel::insert_into(file_source::table)
        .values(&new_file_source)
        .get_result(conn)
        .expect("Error saving new file source")
}
pub fn create_individual_file_row(
    conn: &PgConnection,
    compressed_file_name: String,
    individual_file_name: String,
    instrument: String,
) -> models::MusicFiles {
    use schema::music_files;

    let new_music_files = models::NewMusicFiles {
        compressed_file_name: compressed_file_name.as_str(),
        individual_file_name: individual_file_name.as_str(),
        instrument: instrument.as_str(),
    };

    diesel::insert_into(music_files::table)
        .values(&new_music_files)
        .get_result(conn)
        .expect("Error saving music files")
}
