pub mod download_utils;
pub mod dropbox;
pub mod google_drive;
pub mod mediafire;

use crate::DropboxMetadata;
use crate::GoogleDriveMetadata;
use crate::MediaFireMetadata;

use diesel::pg::PgConnection;

pub trait DownloadFiles<T> {
    fn download(self, hub_conn: Option<&T>, conn: &PgConnection);

    fn metadata_to_sql(self, conn: &PgConnection);
}

#[derive(Debug)]
pub enum DownloadOptions {
    GoogleDrive(GoogleDriveMetadata),
    Dropbox(DropboxMetadata),
    Mediafire(MediaFireMetadata),
}

#[derive(Debug)]
pub struct AssocDataForDownload<DownloadOptions, V> {
    pub download: DownloadOptions,
    pub website_metadata: V,
}
