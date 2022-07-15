table! {
    file_source (id) {
        id -> Integer,
        url -> Text,
        compressed_file_name -> Text,
        time_inserted -> Timestamp,
    }
}

table! {
    music_files (id) {
        id -> Integer,
        compressed_file_name -> Text,
        individual_file_name -> Text,
        instrument -> Text,
    }
}
