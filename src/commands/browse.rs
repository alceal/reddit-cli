use anyhow::Result;

use crate::client::RedditClient;
use crate::format::print_posts_list;
use crate::models::{FormattedPost, Listing, RawPost};
use crate::validation::validate_subreddit;

pub async fn execute(
    client: &RedditClient,
    subreddit: &str,
    sort: &str,
    limit: u32,
    time: &str,
) -> Result<()> {
    let clean = validate_subreddit(subreddit)?;
    let endpoint = format!("/r/{}/{}", clean, sort);

    let limit_str = limit.to_string();
    let time_owned = time.to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", &limit_str)];
    if sort == "top" || sort == "controversial" {
        params.push(("t", &time_owned));
    }

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
