use anyhow::Result;

use crate::client::RedditClient;
use crate::format::print_comments;
use crate::models::{Listing, RawComment, build_comment_tree};
use crate::validation::extract_post_id;

pub async fn execute(
    client: &RedditClient,
    post_id_input: &str,
    sort: &str,
    limit: u32,
) -> Result<()> {
    let extracted = extract_post_id(post_id_input)?;
    let subreddit = match extracted.subreddit {
        Some(s) => s,
        None => client.resolve_subreddit(&extracted.post_id).await?,
    };

    let endpoint = format!("/r/{}/comments/{}", subreddit, extracted.post_id);
    let limit_str = limit.to_string();
    let params: Vec<(&str, &str)> = vec![("sort", sort), ("limit", &limit_str)];

    let data: Vec<serde_json::Value> = client.get(&endpoint, &params).await?;
    let comments_value = data
        .into_iter()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Invalid response from Reddit"))?;

    let comments_listing: Listing<RawComment> = serde_json::from_value(comments_value)?;
    let comments = build_comment_tree(&comments_listing.data.children, 5, limit);

    print_comments(&comments);
    Ok(())
}
