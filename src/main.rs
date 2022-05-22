mod source;
mod storage_download; 

use source::reddit;
use source::reddit::RedditPost;

use storage_download::google_drive::GoogleDriveMetadata; 
use storage_download::google_drive::GoogleFileType; 
use storage_download::google_drive::get_google_drive_connector; 
use storage_download::dropbox::DropboxMetadata; 
use storage_download::mediafire::MediaFireMetadata;
use storage_download::{AssocDataForDownload, DownloadFiles, DownloadOptions};

fn main() {
    let top = reddit::get_posts(100, Some("week"));
    let response = match top {
        Ok(val) => val,
        Err(e) => panic!(
            "There was an issue reading data from reddit with {}. Quitting program.",
            e
        ),
    };

    let vec_basic_list = response.data.children;
    let submission_data_vec: Vec<RedditPost> = vec_basic_list
        .iter()
        .filter_map(|sub| match &sub.data.url {
            Some(url) => Some(RedditPost::new(sub.data.domain.as_str(),
                url.clone(),
                sub.data.subreddit.clone(),
                sub.data.score,
                sub.data.title.clone()
            )),
            None => None,
        })
        .collect();

    let google_drive_hub = get_google_drive_connector().unwrap();
    
    let metadata_and_download_vec: Vec<AssocDataForDownload<DownloadOptions, RedditPost>> =
        submission_data_vec
            .into_iter()
            .filter_map(|post| {
                match post.url_domain {
                    "drive.google.com" =>   Some(AssocDataForDownload {
                        download: DownloadOptions::GoogleDrive(GoogleDriveMetadata::new(
                            post.get_full_url().as_str(),
                            post.get_title(),
                        )),
                        website_metadata: post,
                        
                    }),
                    "mediafire.com" => Some(AssocDataForDownload {
                        download: DownloadOptions::Mediafire(MediaFireMetadata::new(
                            post.get_full_url(),
                        )), 
                        website_metadata: post,
                    }),
                    "dropbox.com" => Some(AssocDataForDownload {
                        download: DownloadOptions::Dropbox(DropboxMetadata::new(
                                post.get_full_url(),
                                post.get_title()
                        )),
                        website_metadata: post,
                    }),
                    _ => None,
                }
            })
            .collect();

    for assoc_data in metadata_and_download_vec.into_iter() {
        match assoc_data.download {
            DownloadOptions::GoogleDrive(val) => {
                match val.file_metadata {
                    Some(GoogleFileType::GoogleFile(_)) => val.download_file(&google_drive_hub),
                    Some(GoogleFileType::GoogleFolder(_)) => (), 
                    _ => ()
                }
            },
            DownloadOptions::Dropbox(val) => val.download(None), 
            DownloadOptions::Mediafire(val) => val.download(None)
        }
    }
}
