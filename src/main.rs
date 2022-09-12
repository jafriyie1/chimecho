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

use anyhow;
use clap::{Parser, Subcommand};
use itertools::izip;
#[macro_use]
extern crate log;

#[derive(Parser, Debug)]
#[clap(
    author = "Joel Afriyie",
    version = "0.1.0",
    about = "Program used to get music sample data from Reddit"
)]
/// Program used to get music sample data from Reddit
struct Cli {
    #[clap(subcommand)]
    cmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    // Download data from Reddit
    Download {
        /// Optional query string for Reddit API. Can get more info here: https://github.com/pushshift/api
        #[clap(short, long)]
        q: Option<String>,
        /// Optional time period. Specified using UTC or day format. Example: --time-period "after=7d"
        /// Example: "after=1586604030&before=1605097230"
        #[clap(short, long)]
        time_period: Option<String>,
        /// Number of steps to iterate over posts list
        #[clap(short, long)]
        step_size: Option<usize>,
        /// File path folder for the music data to live in
        #[clap(short, long)]
        file_path: String,
    },
    // Upload downloaded sample data to GCS
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
) -> anyhow::Result<()> {
    let posts = reddit::get_posts(q, time_period);
    let response = match posts {
        Ok(val) => val,
        Err(e) => panic!(
            "There was an issue reading data from reddit with {}. Quitting program.",
            e
        ),
    };

    let step_size = step_size.unwrap_or(1);

    info!("The file path that was passed from the CLI: {}", &file_path);

    let vec_basic_list: RequestSubmissionResponse = serde_json::from_str(&response)?;
    let vec_basic_list = vec_basic_list.items;
    let submission_data_vec = vec_basic_list.iter().step_by(step_size).filter_map(|sub| {
        sub.url.as_ref().map(|url| {
            RedditPost::new(
                sub.domain.as_str(),
                url.clone(),
                sub.subreddit.clone(),
                sub.score,
                sub.title.clone(),
            )
        })
    });

    let google_drive_hub = get_google_drive_connector()?;

    let metadata_and_download_vec =
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
            });

    let postgres_conn = postgres_orm::establish_connection();
    info!("Downloading music samples from various sources....");

    for assoc_data in metadata_and_download_vec {
        match assoc_data.download {
            DownloadOptions::GoogleDrive(val) => match val.file_metadata {
                Some(GoogleFileType::GoogleFile(_)) => {
                    val.download(Some(&google_drive_hub), &postgres_conn)?;
                }
                _ => (),
            },
            DownloadOptions::Dropbox(val) => val.download(None, &postgres_conn)?,
            DownloadOptions::Mediafire(val) => val.download(None, &postgres_conn)?,
        }
    }

    Ok(())
}

fn upload_to_gcs(file_path: &str, bucket_name: &str) -> anyhow::Result<()> {
    let get_all_sample_path = download_utils::get_files(file_path)?;

    info!(
        "Got all of the uncompressed files from data file path: {}",
        &file_path
    );

    let postgres_conn = postgres_orm::establish_connection();

    for file_obj in get_all_sample_path {
        let temp_file = &file_obj.compressed_file_root;

        let mut music_file_vec = Vec::new();
        // duplicate the file root so that it is the same
        // size as file list for izip op
        let mut compressed_list = Vec::new();
        for _ in &file_obj.file_name_list {
            compressed_list.push(temp_file);
        }

        for (compressed_file_name, individual_file_name, instruments) in izip!(
            compressed_list,
            &file_obj.file_name_list,
            &file_obj.instrument
        ) {
            let new_music_files = postgres_orm::models::NewMusicFiles {
                compressed_file_name,
                individual_file_name,
                instrument: instruments,
            };
            music_file_vec.push(new_music_files);
        }

        debug!(
            "Inserting uncompressed files from file root as a row {} into postgres",
            &temp_file
        );

        debug!(
            "Here are the music files in the vector: {:?}",
            &music_file_vec
        );
        if !music_file_vec.is_empty() {
            postgres_orm::bulk_insert_music_files(&postgres_conn, &music_file_vec)?;
        }
    }

    download_utils::unzip_files(&file_path)?;
    // upload to gcs
    info!("Uploading uncompressed music sample files to GCS.....");
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
    env_logger::init();
    let args = Cli::parse();

    match args.cmd {
        SubCommand::Download {
            q,
            time_period,
            step_size,
            file_path,
        } => match get_zip_music(q, time_period, step_size, file_path) {
            Ok(_) => {}
            Err(e) => error!("error with downloading zip files: {}", e),
        },
        SubCommand::Upload { file_path, bucket } => match upload_to_gcs(&file_path, &bucket) {
            Ok(_) => {}
            Err(e) => error!("error in uploading to gcs: {}", e),
        },
    }
}
