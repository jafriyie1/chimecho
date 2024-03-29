use google_drive3::api::Scope;
use google_drive3::hyper::body::to_bytes;
use google_drive3::{hyper, hyper_rustls, oauth2, DriveHub, Error};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use tokio;
use yup_oauth2;

use crate::DownloadFiles;

use crate::postgres_orm;

#[derive(Debug)]
pub struct GoogleFolder {
    #[allow(dead_code)]
    file_id: String,
    #[allow(dead_code)]
    file_name: String,
}

#[derive(Debug)]
pub struct GoogleFile {
    #[allow(dead_code)]
    file_id: String,
    #[allow(dead_code)]
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

    let secret = yup_oauth2::read_service_account_key(path_to_app_json).await?;
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

    fn get_id(url: &str) -> anyhow::Result<String> {
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

        let regex_func = |cap: Option<Captures>| -> String {
            if let Some(cap_re) = cap {
                let id = cap_re.get(1).map_or("", |m| m.as_str());
                id.to_string()
            } else {
                "".to_string()
            }
        };

        let captured = regex_func(use_re.captures(url));
        Ok(captured)
    }

    pub fn new(url: &str, title: String, file_path: String) -> Self {
        //TODO fix unwrap
        let file_type = Self::file_or_folder(url);
        let file_id = Self::get_id(url).unwrap();

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

        Self {
            id: file_id,
            url: url.to_string(),
            file_metadata,
            file_path,
            out_path: None,
        }
    }
}

impl DownloadFiles<DriveHub> for GoogleDriveMetadata {
    fn metadata_to_sql(self, conn: &diesel::PgConnection) -> anyhow::Result<()> {
        //TODO fix unwrap
        postgres_orm::create_file_row(conn, &self.url, &self.out_path.unwrap())?;

        Ok(())
    }

    #[tokio::main]
    async fn download(
        mut self,
        hub: Option<&DriveHub>,
        conn: &diesel::PgConnection,
    ) -> anyhow::Result<()> {
        //TODO need to handle here
        let resp = hub
            .unwrap()
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

        let data_resp = match resp {
            Ok(val) => Some(val),
            Err(e) => {
                warn!(
                    "Got no response from {} with error response {}. Setting to None",
                    &self.url, e
                );
                None
            }
        };

        debug!(
            "Google drive metadata associated with compressed file: {:?}",
            &self
        );

        if let Some((resp, _)) = data_resp {
            let path_str = format!("{}/{}.zip", &self.file_path, &self.id);
            let path = Path::new(&path_str);
            let display = path.display();

            let mut file = match fs::File::create(&path) {
                Ok(file) => file,
                Err(e) => panic!("couldn't open {}: {}", display, e),
            };

            self.out_path = Some(self.id.clone());

            info!(
                "Name of the compressed file to be saved from google drive: {}",
                &path_str
            );

            let new_response = to_bytes(resp.into_body()).await?;
            file.write_all(&new_response)?;

            info!("Successfully created zip file: {}", &path_str);

            let new_file = fs::File::open(&path)?;
            let new_archive = zip::ZipArchive::new(new_file);
            let new_path_str = path_str.clone().replace(".zip", ".rar");

            if new_archive.is_err() {
                // the file isn't a zip file but it is a RAR file
                // deletes the created zip file and creates a RAR file
                warn!(
                    "Saved zip file {} is actually a rar file. Will save as rar",
                    &path_str
                );
                fs::remove_file(&path_str)?;
                let mut rar_file = fs::File::create(&new_path_str)?;

                self.out_path = Some(self.id.clone());

                rar_file.write_all(&new_response)?;
                info!("Successfully created rar file: {}", &new_path_str);
            }

            self.metadata_to_sql(conn)?;
        }

        Ok(())
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
        assert_eq!("folder", GoogleDriveMetadata::file_or_folder(test_four_url));
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
            GoogleDriveMetadata::get_id(test_one_url).unwrap()
        );
        assert_eq!(
            "1-cgL6_YlB8gOVgoLrwCnP19OqHt34WVj",
            GoogleDriveMetadata::get_id(test_two_url).unwrap()
        );
        assert_eq!(
            "1K4fCarvyqHrkE08H-b2B-fgaOwMRlSkJ",
            GoogleDriveMetadata::get_id(test_three_url).unwrap()
        );
        assert_eq!(
            "1fkzvvlllNowwuZOdlAc0A05p5sZvnsuv",
            GoogleDriveMetadata::get_id(test_four_url).unwrap()
        );
    }
}
