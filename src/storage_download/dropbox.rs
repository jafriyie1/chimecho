use crate::postgres_orm;
use crate::DownloadFiles;
use reqwest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use tokio;

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
    file_path: String,
    out_path: Option<String>,
}

impl DropboxMetadata {
    pub fn new(url: String, file_name: String, file_path: String) -> Self {
        // hack for now
        // roux on url adds amp; and %5C
        // on decoding on string from
        // this has created an issue with
        // getting the right url for
        // dropbox
        let new_url = url.replace("amp;", "");
        let new_url = new_url.replace("%5C", "");
        let new_url = new_url.replace("dl=0", "dl=1");

        let new_file_name = file_name.replace(' ', "_");

        Self {
            url: new_url,
            file_name: new_file_name,
            file_path,
            out_path: None,
        }
    }
}

impl DownloadFiles<String> for DropboxMetadata {
    fn metadata_to_sql(self, conn: &diesel::PgConnection) -> anyhow::Result<()> {
        postgres_orm::create_file_row(conn, &self.url, &self.out_path.unwrap())?;

        Ok(())
    }

    #[tokio::main]
    async fn download(
        mut self,
        _hub: Option<&String>,
        conn: &diesel::PgConnection,
    ) -> anyhow::Result<()> {
        let new_file_name = self.file_name.clone().replace('/', "_");
        let full_file_path = format!("{}/{}.zip", &self.file_path, new_file_name);

        info!(
            "Name of the compressed file to be saved from dropbox: {}",
            &full_file_path
        );

        let path = Path::new(&full_file_path);
        let mut file = match fs::File::create(path) {
            Ok(val) => val,
            Err(e) => panic!("Couldn't open the file: {}", e),
        };

        let resp = reqwest::get(&self.url).await?.bytes().await?;

        file.write_all(&resp)?;
        self.out_path = Some(new_file_name.clone());

        self.metadata_to_sql(conn)?;

        Ok(())
    }
}
