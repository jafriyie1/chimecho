use std::fs;
use std::path::Path;
use std::process::Command;
use zip;

#[derive(Debug)]
pub struct FilesInCompressed {
    pub compressed_file_root: String,
    pub file_name_list: Vec<String>,
    pub instrument: Vec<String>,
}

impl FilesInCompressed {
    fn new(compressed_file_root: String, file_name_list: Vec<String>) -> FilesInCompressed {
        let filter_vec_list = FilesInCompressed::filter_files(file_name_list);
        let instrument_list = FilesInCompressed::get_instrument(&filter_vec_list);

        FilesInCompressed {
            compressed_file_root,
            file_name_list: filter_vec_list,
            instrument: instrument_list,
        }
    }

    fn get_instrument(file_list: &Vec<String>) -> Vec<String> {
        let mut instrument_list = Vec::new();

        let instrument_func = |indi_file_string: &str| {
            if indi_file_string.contains("kick") {
                "kick".to_string()
            } else if indi_file_string.contains("snare") {
                "snare".to_string()
            } else if indi_file_string.contains("hat") {
                "hat".to_string()
            } else if indi_file_string.contains("perc") {
                "perc".to_string()
            } else if indi_file_string.contains("rim") {
                "rim".to_string()
            } else if indi_file_string.contains("clap") {
                "clap".to_string()
            } else if indi_file_string.contains("shaker") {
                "shaker".to_string()
            } else if indi_file_string.contains("ride") {
                "ride".to_string()
            } else if indi_file_string.contains("808") {
                "808".to_string()
            } else if indi_file_string.contains("foley") {
                "foley".to_string()
            } else if indi_file_string.contains("tom") {
                "tom".to_string()
            } else if indi_file_string.contains("fx") {
                "fx".to_string()
            } else if indi_file_string.contains("snap") {
                "snap".to_string()
            } else if indi_file_string.contains("lead") {
                "lead".to_string()
            } else if indi_file_string.contains("pad") {
                "pad".to_string()
            } else if indi_file_string.contains("guitar") {
                "guitar".to_string()
            } else if indi_file_string.contains("piano") {
                "piano".to_string()
            } else if indi_file_string.contains("flute") {
                "flute".to_string()
            } else if indi_file_string.contains("loop") {
                "loop".to_string()
            } else {
                "unspecified".to_string()
            }
        };

        for indi_file in file_list {
            let instrument_from_file = instrument_func(&indi_file.to_lowercase());
            instrument_list.push(instrument_from_file);
        }

        instrument_list
    }

    fn filter_files(file_vec_list: Vec<String>) -> Vec<String> {
        let is_music_file = |file_string: String| {
            if file_string.contains("__MACOSX") || file_string.contains("__macosx") {
                false
            } else {
                file_string.contains(".wav")
                    || file_string.contains(".mp3")
                    || file_string.contains(".flac")
            }
        };

        let final_list: Vec<String> = file_vec_list
            .into_iter()
            .filter_map(|file| match is_music_file(file.clone()) {
                true => Some(file),
                false => None,
            })
            .collect();
        final_list
    }
}

pub fn unzip_files(folder_path: &str) {
    let file_paths = match fs::read_dir(&folder_path) {
        Ok(val) => val,
        Err(e) => panic!(
            "Downloaded files were not saved to directory, so they cannot be read. {}",
            e
        ),
    };

    let mut file_list: Vec<String> = Vec::new();

    for path in file_paths {
        let temp_path = path.unwrap().path().display().to_string();
        if !temp_path.contains("__MACOSX") {
            file_list.push(temp_path);
        }
    }

    // use 7z for unzipping both rar and zip
    for zip_file in file_list {
        let _new_command = Command::new("7z")
            .arg("x")
            .arg(&zip_file)
            .arg("-ounzipped")
            .output()
            .expect("failed to list files in rar.");
    }

    // remove MACOSX directory
    if fs::metadata("./unzipped/__MACOSX/").is_ok() {
        fs::remove_dir_all("./unzipped/__MACOSX/").unwrap();
        info!("removed MACOSX folder from the list of folders that were unzipped");
    }
}

pub fn get_files(folder_path: &str) -> Vec<FilesInCompressed> {
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

        if temp_file_path.contains(".zip") {
            files_in_zip.push(temp_file_path.to_string());
        } else if temp_file_path.contains(".rar") {
            rar_files.push(temp_file_path.to_string())
        }
    }

    let mut all_zip_files: Vec<FilesInCompressed> = files_in_zip
        .into_iter()
        .filter_map(|zipped_file| {
            let temp_path = Path::new(&zipped_file);
            let read_file = fs::File::open(temp_path).unwrap();
            let zip_archive = match zip::ZipArchive::new(read_file) {
                Ok(val) => Some(val),
                Err(_) => None,
            };

            if let Some(temp_zip) = &zip_archive {
                let temp_file_names = zip::ZipArchive::file_names(temp_zip);

                let file_names_to_string: Vec<String> = temp_file_names
                    .into_iter()
                    .map(|file| file.to_string())
                    .collect();
                Some(FilesInCompressed::new(
                    zipped_file.clone(),
                    file_names_to_string,
                ))
            } else {
                None
            }
        })
        .collect();

    let mut all_rar_files: Vec<FilesInCompressed> = rar_files
        .into_iter()
        .map(|rar_file: String| {
            let new_command = Command::new("rar")
                .arg("lb")
                .arg(&rar_file)
                .output()
                .expect("failed to list files in rar.");

            let rar_contents = String::from_utf8(new_command.stdout).unwrap();
            let vec_file_list = rar_contents
                .split('\n')
                .map(|temp_str| temp_str.to_string());
            FilesInCompressed::new(rar_file, vec_file_list.to_owned().collect::<Vec<_>>())
        })
        .collect();

    all_zip_files.append(&mut all_rar_files);

    all_zip_files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_files() {
        let folder_path_one = "./test_samples";
        let vec_list = vec![
            "test/Billie Eilish_Bad Guy (Snap).wav".to_string(),
            "test/Gunna_Idk That Bitch (808).wav".to_string(),
            "test/Kanye West_Broken Road (Snare).wav".to_string(),
            "test/Kanye West_Off The Grid (Hi Hat).wav".to_string(),
            "test/Metro Boomin_Blue Pill (Perc).wav".to_string(),
            "test/Nav_Champion (Kick).wav".to_string(),
            "test/Travis Scott_5% Tint (Rim).wav".to_string(),
            "test/temmmm/Nav_Champion (Kick).wav".to_string(),
        ];
        let comp_files = get_files(folder_path_one);
        let all_files: Vec<String> = comp_files
            .into_iter()
            .map(|val| val.file_name_list)
            .flatten()
            .collect();

        assert!(vec_list.iter().all(|item| all_files.contains(item)));
    }
}
