use anyhow::Result;

use crate::client::RedditClient;
use crate::format::print_posts_list;
use crate::models::{FormattedPost, Listing, RawPost};
use crate::validation::validate_subreddit;

pub async fn execute(
    client: &RedditClient,
    query: &str,
    subreddit: Option<&str>,
    sort: &str,
    limit: u32,
    time: &str,
) -> Result<()> {
    if query.is_empty() || query.len() > 512 {
        anyhow::bail!("Search query must be between 1 and 512 characters");
    }

    let endpoint = if let Some(sub) = subreddit {
        let clean = validate_subreddit(sub)?;
        format!("/r/{}/search", clean)
    } else {
        "/search".to_string()
    };

    let restrict_sr = if subreddit.is_some() {
        "true"
    } else {
        "false"
    };
    let limit_str = limit.to_string();
    let params: Vec<(&str, &str)> = vec![
        ("q", query),
        ("sort", sort),
        ("limit", &limit_str),
        ("t", time),
        ("restrict_sr", restrict_sr),
    ];

    let data: Listing<RawPost> = client.get(&endpoint, &params).await?;
    let posts: Vec<FormattedPost> = data
        .data
        .children
        .into_iter()
        .map(|t| t.data.into())
        .collect();

    print_posts_list(&posts);
    Ok(())
}
