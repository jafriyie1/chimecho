create table file_source (    
    id SERIAL PRIMARY KEY
    url TEXT,
    compressed_file_name TEXT, 
    time_inserted TIMESTAMP
); 

create table music_files (
    id SERIAL PRIMARY KEY, 
    compressed_file_name TEXT, 
    individual_file_name TEXT, 
    instrument TEXT
)