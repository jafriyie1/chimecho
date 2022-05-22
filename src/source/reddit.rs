use roux::responses::{BasicThing, Listing};
use roux::subreddit::responses::submissions::SubmissionsData;
use roux::util::RouxError;
use roux::util::{FeedOption, TimePeriod};
use roux::Subreddit;

#[derive(Debug, Clone)]
pub struct RedditPost<'a> {
    pub url_domain: &'a str,
    full_url: String, 
    subreddit: String,
    score: f64,
    title: String,
}

impl RedditPost<'_> {
    pub fn new(url_domain: &str, full_url: String, subreddit: String, score: f64, title: String) -> RedditPost {
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

#[tokio::main]
pub async fn get_posts(
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

    let hot = subreddit.hot(num_post, reddit_options).await;

    hot
}