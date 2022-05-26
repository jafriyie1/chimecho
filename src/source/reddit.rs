use serde::{Deserialize, Serialize};
use reqwest;

#[derive(Debug, Clone)]
pub struct RedditPost<'a> {
    pub url_domain: &'a str,
    full_url: String,
    subreddit: String,
    score: f64,
    title: String,
}

impl RedditPost<'_> {
    pub fn new(
        url_domain: &str,
        full_url: String,
        subreddit: String,
        score: f64,
        title: String,
    ) -> RedditPost {
        RedditPost {
            url_domain,
            full_url,
            subreddit,
            score,
            title,
        }
    }

    pub fn get_full_url(&self) -> String {
        self.full_url.clone()
    }

    pub fn get_title(&self) -> String {
        self.title.clone()
    }
}

#[derive(serde::Deserialize)]
pub struct SubmissionPost {
    pub domain: String, 
    pub link_flair_text: Option<String>, 
    pub url: Option<String>, 
    pub created_utc: u32,
    pub full_link: String, 
    pub score: f64, 
    pub title: String, 
    pub subreddit: String
}

#[derive(serde::Deserialize)]
pub struct RequestSubmissionResponse {
    #[serde(rename = "data")]
    pub items: Vec<SubmissionPost>
}

#[tokio::main]
pub async fn get_posts(
    q: Option<String>,
    time_period: Option<String>,
) -> Result<String, reqwest::Error> {
    let base_url = String::from("https://api.pushshift.io/reddit/search/submission/?subreddit=drumkits&sort=desc&sort_type=created_utc&size=1000"); 
    
    let base_url = match time_period {
        Some(val) => format!("{}&{}", base_url, val),
        None => base_url
    }; 

    let base_url = match q {
        Some(val) =>  format!("{}&{}", base_url, val),
        None => base_url
    }; 

    reqwest::get(base_url).await.unwrap().text().await
    
}
