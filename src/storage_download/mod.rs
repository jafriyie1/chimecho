pub mod dropbox;
pub mod google_drive;
pub mod mediafire;

use crate::DropboxMetadata;
use crate::GoogleDriveMetadata;
use crate::MediaFireMetadata;

pub trait DownloadFiles<T> {
    fn download(self, resp: T);
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
