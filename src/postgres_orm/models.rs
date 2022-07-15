use super::schema::{file_source, music_files};
use std::time::SystemTime;

#[derive(Insertable)]
#[table_name = "file_source"]
pub struct NewFileSource<'a> {
    pub url: &'a str,
    pub compressed_file_name: &'a str,
    pub time_inserted: &'a SystemTime,
}

#[derive(Queryable)]
pub struct FileSource {
    pub id: i32,
    pub url: String,
    pub compressed_file_name: String,
    pub time_inserted: SystemTime,
}

#[derive(Insertable)]
#[table_name = "music_files"]
pub struct NewMusicFiles<'a> {
    pub compressed_file_name: &'a str,
    pub individual_file_name: &'a str,
    pub instrument: &'a str,
}

#[derive(Queryable)]
pub struct MusicFiles {
    pub id: i32,
    pub compressed_file_name: String,
    pub individual_file_name: String,
    pub instrument: String,
}
