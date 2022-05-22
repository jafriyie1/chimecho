use std::io::Write;
use std::path::Path;
use std::fs;
use tokio;
use soup::prelude::*;
use reqwest;
use reqwest::header::USER_AGENT;
use zip::write::FileOptions;

use crate::DownloadFiles;

#[derive(Debug)]
pub struct MediaFireMetadata {
    url: String,
    raw_html: String, 
}

impl MediaFireMetadata {
    
    pub fn new(url: String) -> MediaFireMetadata {
        let raw_html = MediaFireMetadata::set_html(&url);
        MediaFireMetadata { url, raw_html }
    }


    #[tokio::main]
    async fn set_html(url: &str) -> String {
        let client = reqwest::Client::builder().build().unwrap();
        let response = client.get(url)
                        .header(USER_AGENT, "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:52.0) Gecko/20100101 Firefox/52.0")
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

        let file_name = soup.tag("div")
                                                .attr("class", "filename")
                                                .find()
                                                .unwrap()
                                                .text();
        
        Some(file_name)
    }

    fn get_download_url(&self) -> Option<String> {
        let soup = Soup::new(&self.raw_html);

        let download_url = soup.tag("a")
                                                .attr("class", "popsok")
                                                .find()
                                                .unwrap()
                                                .get("href");

        download_url
    }
}

impl DownloadFiles<Option<String>> for MediaFireMetadata {
    #[tokio::main]
    async fn download(self, _resp: Option<String>) {
        let resp_download_url = self.get_download_url(); 
        let file_name = self.get_file_name().unwrap();
        
        if let Some(download_url) = resp_download_url {
            let path_str = &file_name; 
            let path = Path::new(path_str);
            let file = match fs::File::create(&path) {
                Ok(file) => file, 
                Err(e) => panic!("Couldn't open {}", e)
            };

        let mut zip = zip::ZipWriter::new(file); 
        let options = FileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);

        let resp_content = reqwest::get(&download_url)
                                                                .await
                                                                .unwrap()
                                                                .bytes()
                                                                .await
                                                                .unwrap();
        
        zip.start_file(path_str, options).unwrap(); 
        zip.write_all(&resp_content).unwrap(); 
        zip.finish().unwrap();

        }
    }
}