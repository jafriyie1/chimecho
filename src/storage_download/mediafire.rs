use reqwest;
use reqwest::header::USER_AGENT;
use soup::prelude::*;
use std::fs;
use std::io::Write;
use std::path::Path;
use tokio;

use crate::postgres_orm;
use crate::DownloadFiles;

#[derive(Debug)]
pub struct MediaFireMetadata {
    url: String,
    raw_html: String,
    file_path: String,
    out_path: Option<String>,
}

impl MediaFireMetadata {
    pub fn new(url: String, file_path: String) -> Self {
        //TODO remove unwrap
        let raw_html = Self::set_html(&url).unwrap();
        Self {
            url,
            raw_html,
            file_path,
            out_path: None,
        }
    }

    #[tokio::main]
    async fn set_html(url: &str) -> anyhow::Result<String> {
        let client = reqwest::Client::builder().build()?;
        info!("Getting HTML from Mediafire url: {}", &url);

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
            .await?
            .text()
            .await?;

        Ok(response)
    }

    fn get_file_name(&self) -> Option<String> {
        let soup = Soup::new(&self.raw_html);

        soup.tag("div")
            .attr("class", "filename")
            .find()
            .map(|val| val.text())
    }

    fn get_download_url(&self) -> Option<String> {
        let soup = Soup::new(&self.raw_html);

        let find_url = soup.tag("a").attr("class", "popsok").find();

        match find_url {
            Some(val) => val.get("href"),
            None => {
                warn!("Mediafire url {} is invalid", &self.url);
                None
            }
        }
    }
}

impl DownloadFiles<String> for MediaFireMetadata {
    fn metadata_to_sql(self, conn: &diesel::PgConnection) -> anyhow::Result<()> {
        postgres_orm::create_file_row(conn, &self.url, &self.out_path.unwrap())?;

        Ok(())
    }

    #[tokio::main]
    async fn download(
        mut self,
        _resp: Option<&String>,
        conn: &diesel::PgConnection,
    ) -> anyhow::Result<()> {
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

            let resp_content = reqwest::get(&download_url).await?.bytes().await?;

            file.write_all(&resp_content)?;
            self.out_path = Some(
                original_file_name
                    .clone()
                    .replace(".zip", "")
                    .replace(".rar", ""),
            );

            self.metadata_to_sql(conn)?;
        }
        Ok(())
    }
}
