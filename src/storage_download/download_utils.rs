use std::fs;
use std::path::Path;
use std::process::Command;
use zip;

pub fn get_files(folder_path: &str) -> Vec<String> {
    let base_path = Path::new(folder_path);

    let file_paths = match fs::read_dir(base_path) {
        Ok(val) => val,
        Err(e) => panic!(
            "Downloaded files were not saved to directory, so they cannot be read. {}",
            e
        ),
    };

    let mut files_in_zip = Vec::new();
    let mut rar_files: Vec<String> = Vec::new();

    for path in file_paths {
        let temp_file_path = &path.unwrap().path().display().to_string();

        let temp_path = Path::new(&temp_file_path);
        println!("{:?}", temp_path);
        let read_file = fs::File::open(temp_path).unwrap();
        if temp_file_path.contains(".zip") {
            let zip_archive = match zip::ZipArchive::new(read_file) {
                Ok(val) => Some(val),
                Err(_) => None,
            };
            files_in_zip.push(zip_archive);
        } else if temp_file_path.contains(".rar") {
            rar_files.push(temp_file_path.to_string())
        }
    }

    let mut all_zip_files: Vec<String> = files_in_zip
        .into_iter()
        .filter_map(|zipped_file| match zipped_file {
            Some(zipped_file) => {
                let file_names: Vec<&str> = zip::ZipArchive::file_names(&zipped_file).collect();
                let file_names_to_string: Vec<String> = file_names
                    .into_iter()
                    .map(|file| file.to_string())
                    .collect();
                println!("yoooo");
                Some(file_names_to_string)
            }
            None => None,
        })
        .flatten()
        .collect();

    let mut all_rar_files: Vec<String> = rar_files
        .into_iter()
        .map(|rar_file: String| {
            let new_command = Command::new("rar")
                .arg("lb")
                .arg(&rar_file)
                .output()
                .expect("failed to list files in rar.");

            let rar_contents = String::from_utf8(new_command.stdout).unwrap();
            let vec_file_list = rar_contents
                .split("\n")
                .map(|temp_str| temp_str.to_string());
            vec_file_list.to_owned().collect::<Vec<_>>()
        })
        .flatten()
        .collect();

    all_zip_files.append(&mut all_rar_files);

    let is_music_file = |file_string: String| {
        if file_string.contains(".wav") {
            true
        } else if file_string.contains(".mp3") {
            true
        } else if file_string.contains(".flac") {
            true
        } else {
            false
        }
    };

    let final_list: Vec<String> = all_zip_files
        .into_iter()
        .filter_map(|file| match is_music_file(file.clone()) {
            true => Some(file),
            false => None,
        })
        .collect();
    final_list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_files() {
        let folder_path_one = "./test_samples";
        let mut vec_list = vec![
            "test/Billie Eilish_Bad Guy (Snap).wav".to_string(),
            "test/Full Kit Link.txt".to_string(),
            "test/Gunna_Idk That Bitch (808).wav".to_string(),
            "test/Kanye West_Broken Road (Snare).wav".to_string(),
            "test/Kanye West_Off The Grid (Hi Hat).wav".to_string(),
            "test/Metro Boomin_Blue Pill (Perc).wav".to_string(),
            "test/Nav_Champion (Kick).wav".to_string(),
            "test/Travis Scott_5% Tint (Rim).wav".to_string(),
            "test/temmmm/Nav_Champion (Kick).wav".to_string(),
        ];
        assert_eq!(get_files(folder_path_one).sort(), vec_list.sort());
    }
}
