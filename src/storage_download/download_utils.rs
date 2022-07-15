use std::fs;
use std::path::Path;
use std::process::Command;
use zip::{self, ZipArchive};

#[derive(Debug)]
pub struct FilesInCompressed {
    pub compressed_file_root: Vec<String>,
    pub file_name_list: Vec<String>,
    pub instrument: Vec<String>,
}

impl FilesInCompressed {
    fn new(compressed_file_root: String, file_name_list: Vec<String>) -> FilesInCompressed {
        let filter_vec_list = FilesInCompressed::filter_files(file_name_list);
        let instrument_list = FilesInCompressed::get_instrument(&filter_vec_list);
        let mut compressed_list = Vec::new();
        for _ in &filter_vec_list {
            compressed_list.push(compressed_file_root.clone());
        }
        FilesInCompressed {
            compressed_file_root: compressed_list,
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
            let instrument_from_file = instrument_func(&indi_file);
            instrument_list.push(instrument_from_file);
        }

        instrument_list
    }

    fn filter_files(file_vec_list: Vec<String>) -> Vec<String> {
        let is_music_file = |file_string: String| {
            if file_string.contains(".wav") {
                true
            } else if file_string.contains(".mp3") {
                true
            } else if file_string.contains(".flac") {
                true
            } else if file_string.contains("__MACOSX") {
                false
            } else if file_string.contains("__macosx") {
                false
            } else {
                false
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

        //println!("{:?}", temp_path);
        if temp_file_path.contains(".zip") {
            files_in_zip.push(temp_file_path.to_string());
        } else if temp_file_path.contains(".rar") {
            rar_files.push(temp_file_path.to_string())
        }
    }

    let mut all_zip_files: Vec<FilesInCompressed> = files_in_zip
        .into_iter()
        .map(|zipped_file| {
            let temp_path = Path::new(&zipped_file);
            let read_file = fs::File::open(temp_path).unwrap();
            let zip_archive = match zip::ZipArchive::new(read_file) {
                Ok(val) => Some(val),
                Err(_) => None,
            };

            if let Some(temp_zip) = &zip_archive {
                let temp_file_names = zip::ZipArchive::file_names(temp_zip);

                let file_names_zip: Vec<&str> = temp_file_names.collect();
                let file_names_to_string: Vec<String> = file_names_zip
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
        .filter_map(|items| match items {
            Some(val) => Some(val),
            None => None,
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
                .split("\n")
                .map(|temp_str| temp_str.to_string());
            FilesInCompressed::new(
                rar_file.clone(),
                vec_file_list.to_owned().collect::<Vec<_>>(),
            )
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
        //assert_eq!(get_files(folder_path_one).sort(), vec_list.sort());
    }
}
