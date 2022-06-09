mod source;
mod storage_download;

use source::reddit;
use source::reddit::{RedditPost, RequestSubmissionResponse};

use storage_download::download_utils;
use storage_download::dropbox::DropboxMetadata;
use storage_download::google_drive::get_google_drive_connector;
use storage_download::google_drive::GoogleDriveMetadata;
use storage_download::google_drive::GoogleFileType;
use storage_download::mediafire::MediaFireMetadata;
use storage_download::{AssocDataForDownload, DownloadFiles, DownloadOptions};

use clap::Parser;
use serde_json;

#[clap(
    author = "Joel Afriyie",
    version = "0.1.0",
    about = "Program used to get music sample data from Reddit"
)]
/// Program used to get music sample data from Reddit
#[derive(Parser, Default, Debug)]
pub struct CLI {
    /// Optional query string for Reddit API
    #[clap(short, long)]
    q: Option<String>,
    /// Optional time period. Example: after=7d
    #[clap(short, long)]
    time_period: Option<String>,
    /// Number of steps to iterate over posts list
    #[clap(short, long)]
    step_size: Option<usize>,
    /// File path folder for the music data to live in
    #[clap(short, long)]
    file_path: String,
}

fn main() {
    let args = CLI::parse();
    let posts = reddit::get_posts(args.q.clone(), args.time_period.clone());
    let response = match posts {
        Ok(val) => val,
        Err(e) => panic!(
            "There was an issue reading data from reddit with {}. Quitting program.",
            e
        ),
    };

    let step_size = match args.step_size {
        Some(val) => val,
        None => 1,
    };
    println!("yooo here is the file path: {}", &args.file_path);

    let vec_basic_list: RequestSubmissionResponse = serde_json::from_str(&response).unwrap();
    let vec_basic_list = vec_basic_list.items;
    let submission_data_vec: Vec<RedditPost> = vec_basic_list
        .iter()
        .step_by(step_size)
        .filter_map(|sub| match &sub.url {
            Some(url) => Some(RedditPost::new(
                sub.domain.as_str(),
                url.clone(),
                sub.subreddit.clone(),
                sub.score,
                sub.title.clone(),
            )),
            None => None,
        })
        .collect();

    let google_drive_hub = get_google_drive_connector().unwrap();

    let metadata_and_download_vec: Vec<AssocDataForDownload<DownloadOptions, RedditPost>> =
        submission_data_vec
            .into_iter()
            .filter_map(|post| match post.url_domain {
                "drive.google.com" => Some(AssocDataForDownload {
                    download: DownloadOptions::GoogleDrive(GoogleDriveMetadata::new(
                        post.get_full_url().as_str(),
                        post.get_title(),
                        args.file_path.clone(),
                    )),
                    website_metadata: post,
                }),
                "mediafire.com" => Some(AssocDataForDownload {
                    download: DownloadOptions::Mediafire(MediaFireMetadata::new(
                        post.get_full_url(),
                        args.file_path.clone(),
                    )),
                    website_metadata: post,
                }),
                "dropbox.com" => Some(AssocDataForDownload {
                    download: DownloadOptions::Dropbox(DropboxMetadata::new(
                        post.get_full_url(),
                        post.get_title(),
                        args.file_path.clone(),
                    )),
                    website_metadata: post,
                }),
                _ => None,
            })
            .collect();

    for assoc_data in metadata_and_download_vec.into_iter() {
        match assoc_data.download {
            DownloadOptions::GoogleDrive(val) => match val.file_metadata {
                Some(GoogleFileType::GoogleFile(_)) => val.download_file(&google_drive_hub),
                Some(GoogleFileType::GoogleFolder(_)) => (),
                _ => (),
            },
            DownloadOptions::Dropbox(val) => val.download(None),
            DownloadOptions::Mediafire(val) => val.download(None),
        }
    }
    let get_all_sample_path = download_utils::get_files(&args.file_path.clone());
    println!("Got all of the files {:?}", get_all_sample_path);
}
