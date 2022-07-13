use google_drive3::api::Scope;
use google_drive3::api::{File, FileList};
use google_drive3::hyper::body::to_bytes;
use google_drive3::hyper::Body;
use google_drive3::hyper::Response;
use google_drive3::{hyper, hyper_rustls, oauth2, DriveHub, Error};
use lazy_static::lazy_static;
use regex::Regex;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use tokio;
use yup_oauth2;
use zip::write::FileOptions;

use crate::DownloadFiles;

#[derive(Debug)]
pub struct GoogleFolder {
    file_id: String,
    file_name: String,
}

#[derive(Debug)]
pub struct GoogleFile {
    file_id: String,
    file_name: String,
}

#[derive(Debug)]
pub enum GoogleFileType {
    GoogleFolder(GoogleFolder),
    GoogleFile(GoogleFile),
}

#[derive(Debug)]
pub struct GoogleDriveMetadata {
    id: String,
    url: String,
    pub file_metadata: Option<GoogleFileType>,
    file_path: String,
    out_path: Option<String>,
}

#[tokio::main]
pub async fn get_google_drive_connector() -> Result<DriveHub, Error> {
    let path_to_app_json = match env::var("GOOGLE_APPLICATION_CREDENTIALS") {
        Ok(val) => val,
        Err(e) => panic!(
            "The google application credentials couldn't be read. {} . Exiting.",
            e
        ),
    };

    let secret = yup_oauth2::read_service_account_key(path_to_app_json)
        .await
        .unwrap();
    let auth_result = oauth2::ServiceAccountAuthenticator::builder(secret)
        .build()
        .await;

    let auth = match auth_result {
        Ok(val) => val,
        Err(e) => panic!(
            "Authentication for Google Drive failed with reason: {}. Quitting program.",
            e
        ),
    };

    let hub = DriveHub::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .enable_http2()
                .build(),
        ),
        auth,
    );

    Ok(hub)
}

impl GoogleDriveMetadata {
    fn file_or_folder(url: &str) -> &str {
        if url.contains("folder") {
            "folder"
        } else if url.contains("file") {
            "file"
        } else {
            "other"
        }
    }

    #[tokio::main]
    pub async fn download_files_from_folder(
        folder_id: &str,
        hub: &DriveHub,
    ) -> Result<(Response<Body>, FileList), Error> {
        // recurse on result
        // if result is not a folder
        // add to list
        let result = hub
            .files()
            .list()
            .supports_all_drives(true)
            .include_items_from_all_drives(true)
            .q(&format!("'{}' in parents", folder_id))
            .doit()
            .await;

        result
    }

    #[tokio::main]
    pub async fn download_file(self, hub: &DriveHub) {
        let file = hub
            .files()
            .get(self.id.as_str())
            .param("alt", "media")
            .supports_team_drives(true)
            .supports_all_drives(true)
            .include_permissions_for_view("published")
            .acknowledge_abuse(false)
            .add_scope(Scope::Full)
            .doit()
            .await;

        tokio::task::spawn_blocking(|| {
            GoogleDriveMetadata::download(self, file);
        })
        .await
        .expect("Task panicked")
    }

    fn get_id(url: &str) -> String {
        lazy_static! {
            static ref RE: Regex = Regex::new(
                "https://drive.google.com/file/d/([a-zA-z0-9-]+)([/view]+)?.?usp=[a-zA-Z]*"
            )
            .unwrap();
            static ref RE_TWO: Regex =
                Regex::new("https://drive.google.com/drive/folders/([a-zA-z0-9-]+).?usp=[a-zA-Z]*")
                    .unwrap();
            static ref RE_THREE: Regex =
                Regex::new("https://drive.google.com/drive[/a-zA-z/]+?folders/([a-zA-z0-9-]+)")
                    .unwrap();
            static ref RE_FOUR: Regex =
                Regex::new("https://drive.google.com/file/d/([a-zA-z0-9-]+)/view").unwrap();
            static ref RE_FIVE: Regex =
                Regex::new("https://drive.google.com/drive/u/[0-9]/folders/([a-zA-z0-9-]+)")
                    .unwrap();
            static ref REGEXS: Vec<&'static Regex> =
                vec![&RE, &RE_TWO, &RE_THREE, &RE_FOUR, &RE_FIVE];
        }

        // set to first regex
        let mut use_re = &*REGEXS[0];
        for regex in &*REGEXS {
            if regex.is_match(url) {
                use_re = regex;
            }
        }

        println!("{:?}", url);
        let captured = use_re.captures(url).unwrap();
        let id = captured.get(1).map_or("", |m| m.as_str());
        id.to_string()
    }

    pub fn new(url: &str, title: String, file_path: String) -> GoogleDriveMetadata {
        let file_type = GoogleDriveMetadata::file_or_folder(url);
        let file_id = GoogleDriveMetadata::get_id(url);

        let file_metadata = match file_type {
            "file" => Some(GoogleFileType::GoogleFile(GoogleFile {
                file_id: file_id.clone(),
                file_name: title,
            })),
            "folder" => Some(GoogleFileType::GoogleFolder(GoogleFolder {
                file_id: file_id.clone(),
                file_name: title,
            })),
            _ => None,
        };

        GoogleDriveMetadata {
            id: file_id,
            url: url.to_string(),
            file_metadata,
            file_path,
            out_path: None,
        }
    }
}

impl DownloadFiles<Result<(Response<Body>, File), Error>> for GoogleDriveMetadata {
    #[tokio::main]
    async fn download(mut self, resp: Result<(Response<Body>, File), Error>) {
        let data_resp = match resp {
            Ok(val) => Some(val),
            Err(_) => None,
        };

        if let Some((resp, google_file)) = data_resp {
            let path_str = format!("{}/{}.zip", &self.file_path, &self.id);
            let path = Path::new(&path_str);
            let display = path.display();
            let mut file = match fs::File::create(&path) {
                Ok(file) => file,
                Err(e) => panic!("couldn't open {}: {}", display, e),
            };

            self.out_path = Some(self.id.clone());

            println!("Here is the file path google: {}", &path_str);

            //let mut zip = zip::ZipWriter::new(file);

            //let options = FileOptions::default().compression_method(zip::CompressionMethod::Bzip2);

            let new_response = to_bytes(resp.into_body()).await.unwrap();

            let file_name = format!("{}.zip", &self.id);
            //zip.start_file(&file_name, options).unwrap();
            file.write_all(&new_response).unwrap();
            //zip.finish().unwrap();

            let new_file = fs::File::open(&path).unwrap();

            let mut new_archive = zip::ZipArchive::new(new_file);

            let new_path_str = path_str.clone().replace(".zip", ".rar");
            if let Err(files) = new_archive {
                fs::remove_file(&path_str).unwrap();
                //let new_file = OpenOptions::new().write(true).open(&new_path_str);

                let mut rar_file = fs::File::create(&new_path_str).unwrap();

                self.out_path = Some(self.id.clone());

                rar_file.write_all(&new_response).unwrap();
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_or_folder() {
        let test_one_url =
            "https://drive.google.com/drive/folders/1Ny62TwY-Rgz4cfQDwcdBHL0vtWJgy6DI";
        let test_two_url = "https://drive.google.com/file/d/1-cgL6_YlB8gOVgoLrwCnP19OqHt34WVj/view";
        let test_three_url =
            "https://www.dropbox.com/sh/hkgtorveen2jvh6/AAAf0TStSQD_9PAOTjubPU1Ma?dl=0";
        let test_four_url =
            "https://drive.google.com/drive/u/4/folders/1Xw-HoupNY75aYB1Hc0zLifFxu3g5RQGX";

        assert_eq!("folder", GoogleDriveMetadata::file_or_folder(test_one_url));
        assert_eq!("file", GoogleDriveMetadata::file_or_folder(test_two_url));
        assert_eq!("other", GoogleDriveMetadata::file_or_folder(test_three_url));
    }

    #[test]
    fn test_get_id() {
        let test_one_url =
            "https://drive.google.com/drive/folders/1Ny62TwY-Rgz4cfQDwcdBHL0vtWJgy6DI";
        let test_two_url = "https://drive.google.com/file/d/1-cgL6_YlB8gOVgoLrwCnP19OqHt34WVj/view";
        let test_three_url =
            "https://drive.google.com/file/d/1K4fCarvyqHrkE08H-b2B-fgaOwMRlSkJ/view";
        let test_four_url =
            "https://drive.google.com/file/d/1fkzvvlllNowwuZOdlAc0A05p5sZvnsuv/view";

        assert_eq!(
            "1Ny62TwY-Rgz4cfQDwcdBHL0vtWJgy6DI",
            GoogleDriveMetadata::get_id(test_one_url)
        );
        assert_eq!(
            "1-cgL6_YlB8gOVgoLrwCnP19OqHt34WVj",
            GoogleDriveMetadata::get_id(test_two_url)
        );
        assert_eq!(
            "1K4fCarvyqHrkE08H-b2B-fgaOwMRlSkJ",
            GoogleDriveMetadata::get_id(test_three_url)
        );
        assert_eq!(
            "1fkzvvlllNowwuZOdlAc0A05p5sZvnsuv",
            GoogleDriveMetadata::get_id(test_four_url)
        );
    }
}
