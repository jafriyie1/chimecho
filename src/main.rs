#[macro_use]
extern crate diesel;
mod postgres_orm;
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

use clap::{Parser, Subcommand};
use serde_json;

use itertools::izip;

#[derive(Parser, Debug)]
#[clap(
    author = "Joel Afriyie",
    version = "0.1.0",
    about = "Program used to get music sample data from Reddit"
)]
/// Program used to get music sample data from Reddit
struct CLI {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    // Download data from reddit
    Download {
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
    },
    Upload {
        /// File path folder that contains zip and rar files
        #[clap(short, long)]
        file_path: String,
        /// bucket name for google cloud storage upload
        #[clap(short, long)]
        bucket: String,
    },
}

fn get_zip_music(
    q: Option<String>,
    time_period: Option<String>,
    step_size: Option<usize>,
    file_path: String,
) -> std::io::Result<()> {
    let posts = reddit::get_posts(q, time_period);
    let response = match posts {
        Ok(val) => val,
        Err(e) => panic!(
            "There was an issue reading data from reddit with {}. Quitting program.",
            e
        ),
    };

    let step_size = match step_size {
        Some(val) => val,
        None => 1,
    };
    println!("yooo here is the file path: {}", &file_path);

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
                        file_path.clone(),
                    )),
                    website_metadata: post,
                }),
                "mediafire.com" => Some(AssocDataForDownload {
                    download: DownloadOptions::Mediafire(MediaFireMetadata::new(
                        post.get_full_url(),
                        file_path.clone(),
                    )),
                    website_metadata: post,
                }),
                "dropbox.com" => Some(AssocDataForDownload {
                    download: DownloadOptions::Dropbox(DropboxMetadata::new(
                        post.get_full_url(),
                        post.get_title(),
                        file_path.clone(),
                    )),
                    website_metadata: post,
                }),
                _ => None,
            })
            .collect();

    let postgres_conn = postgres_orm::establish_connection();
    for assoc_data in metadata_and_download_vec.into_iter() {
        match assoc_data.download {
            DownloadOptions::GoogleDrive(val) => match val.file_metadata {
                Some(GoogleFileType::GoogleFile(_)) => {
                    val.download(Some(&google_drive_hub), &postgres_conn)
                }
                Some(GoogleFileType::GoogleFolder(_)) => (),
                _ => (),
            },
            DownloadOptions::Dropbox(val) => val.download(None, &postgres_conn),
            DownloadOptions::Mediafire(val) => val.download(None, &postgres_conn),
        }
    }

    //for assoc_data in metadata_and_download_vec.into_iter() {
    //    match assoc_data.download {
    //        DownloadOptions::GoogleDrive(val) => match val.file_metadata {
    //            Some(GoogleFileType::GoogleFile(_)) => val.metadata_to_sql(&postgres_conn),
    //            Some(GoogleFileType::GoogleFolder(_)) => (),
    //            _ => (),
    //        },
    //        DownloadOptions::Dropbox(val) => val.metadata_to_sql(&postgres_conn),
    //        DownloadOptions::Mediafire(val) => val.metadata_to_sql(&postgres_conn),
    //    }
    //}
    Ok(())
}

fn upload_to_gcs(file_path: String, bucket_name: String) -> std::io::Result<()> {
    let get_all_sample_path = download_utils::get_files(&file_path.clone());
    //println!("Got all of the files {:?}", get_all_sample_path);
    let postgres_conn = postgres_orm::establish_connection();

    for file_obj in get_all_sample_path {
        let temp_file = &file_obj.compressed_file_root;

        for (file_root, all_files, instruments) in
            izip!(temp_file, &file_obj.file_name_list, &file_obj.instrument)
        {
            postgres_orm::create_individual_file_row(
                &postgres_conn,
                file_root.to_string(),
                all_files.to_string(),
                instruments.to_string(),
            );
        }
    }

    download_utils::unzip_files(&file_path.clone());
    // upload to gcs
    let _new_command = std::process::Command::new("gsutil")
        .arg("-m")
        .arg("cp")
        .arg("-r")
        .arg("-n")
        .arg("./unzipped/")
        .arg(format!("gs://{}", bucket_name).as_str())
        .output()
        .expect("failed to list files in rar.");

    Ok(())
}

fn main() {
    let args = CLI::parse();

    match args.cmd {
        SubCommand::Download {
            q,
            time_period,
            step_size,
            file_path,
        } => match get_zip_music(q, time_period, step_size, file_path.clone()) {
            Ok(_) => {}
            Err(e) => eprintln!("error with downloading zip files: {}", e),
        },
        SubCommand::Upload { file_path, bucket } => {
            match upload_to_gcs(file_path.clone(), bucket.clone()) {
                Ok(_) => {}
                Err(e) => eprintln!("error in uploading to gcs: {}", e),
            }
        }
    }
}
