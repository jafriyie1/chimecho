use reqwest;
use reqwest::header::AUTHORIZATION;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use tokio;
use zip::write::FileOptions;

use crate::DownloadFiles;

#[derive(Serialize, Deserialize, Debug)]
pub struct DropboxAudienceOptions {
    allowed: bool,
    audience: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropboxPolicyOptions {
    allowed: bool,
    policy: HashMap<String, String>,
    resolved_policy: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropboxLinkPermissions {
    all_comments: bool,
    allow_download: bool,
    audience_options: Vec<DropboxAudienceOptions>,
    can_allow_download: bool,
    can_disallow_download: bool,
    can_remove_expiry: bool,
    can_remove_password: bool,
    can_revoke: bool,
    can_set_expiry: bool,
    can_set_password: bool,
    can_use_extended_sharing_controls: bool,
    require_password: bool,
    resolved_visibility: HashMap<String, String>,
    revoke_failure_reason: HashMap<String, String>,
    team_restrics_comments: bool,
    visibility_policies: Vec<DropboxAudienceOptions>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropboxTeamMemberInfo {
    display_name: String,
    member_id: String,
    team_info: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropboxDownloadResponse {
    #[serde(rename = "tag")]
    tag: String,
    client_modified: String,
    id: String,
    link_permissions: DropboxLinkPermissions,
    name: String,
    path_lower: String,
    rev: String,
    server_modified: String,
    size: f64,
    team_member_info: DropboxTeamMemberInfo,
    url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DropboxMetadata {
    file_name: String,
    url: String,
    file_path: String
}

impl DropboxMetadata {
    pub fn new(url: String, file_name: String, file_path: String) -> DropboxMetadata {
        // hack for now
        // roux on url adds amp; and %5C
        // on decoding on string from
        // &
        // this has created an issue with
        // getting the right url for
        // dropbox
        let new_url = url.replace("amp;", "");
        let new_url = new_url.replace("%5C", "");
        let new_url = new_url.replace("dl=0", "dl=1");

        let new_file_name = file_name.replace(' ', "_");

        DropboxMetadata {
            url: new_url,
            file_name: new_file_name,
            file_path
        }
    }

    #[tokio::main]
    async fn get_download_url_and_file(&self) -> (Option<String>, Option<String>) {
        let client = reqwest::Client::builder().build().unwrap();

        let endpoint = "https://content.dropboxapi.com/2/sharing/get_shared_link_file";
        let json_val = serde_json::to_string(self).unwrap();
        let token = format!("Bearer {}", env::var("DROPBOX_APP_TOKEN").unwrap());
        let resp = client
            .post(endpoint)
            .header(AUTHORIZATION, token)
            .header("Dropbox-API-Arg", &json_val)
            .send()
            .await
            .unwrap();

        let json_response: DropboxDownloadResponse =
            serde_json::from_str(&resp.text().await.unwrap()).unwrap();

        let download_url = Some(json_response.url);
        let file_name = Some(json_response.name);

        (download_url, file_name)
    }
}

impl DownloadFiles<Option<String>> for DropboxMetadata {
    #[tokio::main]
    async fn download(self, _resp: Option<String>) {
        let file_name = format!("{}/{}.zip", &self.file_path, &self.file_name);
        let path = Path::new(&file_name);
        let file = match fs::File::create(path) {
            Ok(val) => val,
            Err(e) => panic!("Couldn't open the file: {}", e),
        };

        let resp = reqwest::get(&self.url)
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();

        let mut zip = zip::ZipWriter::new(file);
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file(&file_name, options).unwrap();
        zip.write_all(&resp).unwrap();
        zip.finish().unwrap();
    }
}
