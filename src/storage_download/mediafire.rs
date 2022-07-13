use reqwest;
use reqwest::header::USER_AGENT;
use soup::prelude::*;
use std::fs;
use std::io::Write;
use std::path::Path;
use tokio;
use zip::write::FileOptions;

use crate::DownloadFiles;

#[derive(Debug)]
pub struct MediaFireMetadata {
    url: String,
    raw_html: String,
    file_path: String,
    out_path: Option<String>,
}

impl MediaFireMetadata {
    pub fn new(url: String, file_path: String) -> MediaFireMetadata {
        let raw_html = MediaFireMetadata::set_html(&url);
        MediaFireMetadata {
            url,
            raw_html,
            file_path,
            out_path: None,
        }
    }

    #[tokio::main]
    async fn set_html(url: &str) -> String {
        let client = reqwest::Client::builder().build().unwrap();
        let response = client
            .get(url)
            .header(
                USER_AGENT,
                "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:52.0) Gecko/20100101 Firefox/52.0",
            )
            .header("Access-Control-Max-Age", "3600")
            .header("Access-Control-Allow-Headers", "Content-Type")
            .header("Access-Control-Allow-Methods", "GET")
            .header("Access-Control-Allow-Origin", "*")
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        response
    }

    fn get_file_name(&self) -> Option<String> {
        let soup = Soup::new(&self.raw_html);

        let find_file_name = soup.tag("div").attr("class", "filename").find();

        let file_name = match find_file_name {
            Some(val) => Some(val.text()),
            None => None,
        };

        file_name
    }

    fn get_download_url(&self) -> Option<String> {
        let soup = Soup::new(&self.raw_html);

        let find_url = soup.tag("a").attr("class", "popsok").find();

        let download_url = match find_url {
            Some(val) => val.get("href"),
            None => None,
        };

        download_url
    }
}

impl DownloadFiles<Option<String>> for MediaFireMetadata {
    #[tokio::main]
    async fn download(mut self, _resp: Option<String>) {
        let resp_download_url = self.get_download_url();
        let resp_file_name = self.get_file_name();

        if let (Some(download_url), Some(original_file_name)) = (resp_download_url, resp_file_name)
        {
            let file_name = format!("{}/{}", &self.file_path, &original_file_name);
            let path_str = &file_name;

            let path = Path::new(path_str);
            let mut file = match fs::File::create(&path) {
                Ok(file) => file,
                Err(e) => panic!("Couldn't open {}", e),
            };

            let resp_content = reqwest::get(&download_url)
                .await
                .unwrap()
                .bytes()
                .await
                .unwrap();

            file.write_all(&resp_content).unwrap();
            self.out_path = Some(
                original_file_name
                    .clone()
                    .replace(".zip", "")
                    .replace(".rar", ""),
            );
        }
    }
}
