use google_drive3::api::Scope;
use google_drive3::api::{File, FileList};
use google_drive3::hyper::{Body, Response};
use google_drive3::{hyper, hyper_rustls, oauth2, DriveHub, Error};
use lazy_static::lazy_static;
use regex::Regex;
use roux::responses::{BasicThing, Listing};
use roux::subreddit::responses::submissions::SubmissionsData;
use roux::util::RouxError;
use roux::util::{FeedOption, TimePeriod};
use roux::Subreddit;
use std::env;
use yup_oauth2;

#[tokio::main]
async fn reddit(
    num_post: u32,
    time_period: Option<&str>,
) -> Result<BasicThing<Listing<BasicThing<SubmissionsData>>>, RouxError> {
    let subreddit = Subreddit::new("Drumkits");

    let reddit_options: Option<FeedOption> = match time_period {
        Some("now") => Some(FeedOption::new().period(TimePeriod::Now)),
        Some("today") => Some(FeedOption::new().period(TimePeriod::Today)),
        Some("week") => Some(FeedOption::new().period(TimePeriod::ThisWeek)),
        Some("month") => Some(FeedOption::new().period(TimePeriod::ThisMonth)),
        Some("year") => Some(FeedOption::new().period(TimePeriod::ThisYear)),
        Some("all") => Some(FeedOption::new().period(TimePeriod::AllTime)),
        Some(_) => None,
        None => None,
    };

    let hot = subreddit.top(num_post, reddit_options).await;

    hot
}

#[derive(Debug)]
struct GoogleDriveMetadata {
    id: String,
    url: String,
    file_type: String,
    file_content: GoogleFiles,
}

#[derive(Debug)]
enum GoogleFiles {
    Folder((Response<Body>, File)),
    FileList(Option<(Response<Body>, FileList)>),
}

#[tokio::main]
async fn get_google_drive_connector() -> DriveHub {
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

    hub
}

impl GoogleDriveMetadata {
    fn file_or_folder(url: &str) -> &str {
        let file_type = if url.contains("folder") {
            "folder"
        } else if url.contains("file") {
            "file"
        } else {
            "other"
        };

        file_type
    }

    #[tokio::main]
    async fn download_files_from_folder(
        folder_id: &str,
        hub: &DriveHub,
    ) -> Result<(Response<Body>, FileList), Error> {
        let result = hub.files()
                                                        .list()
                                                        .supports_all_drives(true)
                                                        .include_items_from_all_drives(true)
                                                        .q(&format!("'{}' in parents", folder_id))
                                                        //.q("mimeType = 'application/vnd.google-apps.folder'")
                                                        .doit().await;

        //println!("{:?}", result.unwrap());

        result
    }

    #[tokio::main]
    async fn download_file(id: &str, hub: &DriveHub) -> Result<(Response<Body>, File), Error> {
        println!("got the conn");

        let file = hub
            .files()
            .get(id)
            .param("alt", "media")
            .supports_team_drives(true)
            .supports_all_drives(true)
            .include_permissions_for_view("published")
            .acknowledge_abuse(false)
            .add_scope(Scope::Full)
            .doit()
            .await;

        file
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
            static ref REGEXS: Vec<&'static Regex> = vec![&RE, &RE_TWO, &RE_THREE, &RE_FOUR];
        }

        // set to first regex
        let mut use_re = &*REGEXS[0];
        for regex in &*REGEXS {
            if regex.is_match(url) {
                use_re = regex;
            }
        }

        println!("{}", url);
        let captured = use_re.captures(url).unwrap();
        let id = captured.get(1).map_or("", |m| m.as_str());
        return id.to_string();
    }

    fn new(url: &str, hub: &DriveHub) -> GoogleDriveMetadata {
        let file_type = GoogleDriveMetadata::file_or_folder(&url);
        let file_id = GoogleDriveMetadata::get_id(&url);

        let file_data = match file_type {
            "file" => {
                GoogleFiles::Folder(GoogleDriveMetadata::download_file(&file_id, hub).unwrap())
            }
            "folder" => GoogleFiles::FileList(Some(
                    GoogleDriveMetadata::download_files_from_folder(&file_id, hub).unwrap(),
                )),
            _ => GoogleFiles::FileList(None),
        };

        // ,
        //GoogleFiles::FileList(GoogleDriveMetadata::download_files_from_folder(&file_id, hub).unwrap())

        let m = GoogleDriveMetadata {
            id: file_id.clone(),
            url: url.to_owned(),
            file_type: file_type.to_string(),
            file_content: file_data,
        };

        println!("{:?}", m);
        m
    }
}

#[derive(Debug)]
pub struct DropboxMetadata {
    id: String,
    url: String,
}

#[derive(Debug)]
pub enum DownloadOptions {
    GoogleDriveMetadata,
    DropboxMetadata,
}

#[derive(Debug)]
struct AssocDataForDownload<'a, T: 'a, V> {
    website_metadata: &'a T,
    download: V,
}

#[derive(Debug, Clone)]
struct RedditPost<'a> {
    url_domain: &'a str,
    full_url: Option<String>,
    subreddit: String,
    score: f64,
    title: String,
}

fn main() {
    let top = reddit(1, Some("year"));
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
        .filter_map(|sub| match sub.data.url {
            Some(_) => Some(RedditPost {
                url_domain: sub.data.domain.as_str(),
                full_url: sub.data.url.clone(),
                subreddit: sub.data.subreddit.clone(),
                score: sub.data.score,
                title: sub.data.title.clone(),
            }),
            None => None,
        })
        .collect();

    //let temp = submission_data_vec.as_slice();
    println!("{:?}", submission_data_vec.len());
    //Vec<AssocDataForDownload<RedditPost>>
    let google_drive_hub = get_google_drive_connector();
    let metadata_and_download_vec: Vec<AssocDataForDownload<RedditPost, GoogleDriveMetadata>> =
        submission_data_vec
            .iter()
            .filter_map(|post| {
                match post.url_domain {
                    "drive.google.com" => Some(AssocDataForDownload {
                        website_metadata: post,
                        download: GoogleDriveMetadata::new(
                            post.full_url.clone().unwrap().as_str(),
                            &google_drive_hub,
                        ),
                    }),
                    // if you see _
                    _ => None,
                }
            })
            .collect();

    println!("{:?}", metadata_and_download_vec);
    println!("{}", metadata_and_download_vec.len());
}
