use anyhow::Result;

use crate::client::RedditClient;
use crate::format::print_post_detail;
use crate::models::{FormattedPost, Listing, RawComment, RawPost, build_comment_tree};
use crate::validation::extract_post_id;

pub async fn execute(
    client: &RedditClient,
    post_id_input: &str,
    depth: u32,
    limit: u32,
) -> Result<()> {
    let extracted = extract_post_id(post_id_input)?;
    let subreddit = match extracted.subreddit {
        Some(s) => s,
        None => client.resolve_subreddit(&extracted.post_id).await?,
    };

    let endpoint = format!("/r/{}/comments/{}", subreddit, extracted.post_id);
    let depth_str = depth.to_string();
    let limit_str = limit.to_string();
    let params: Vec<(&str, &str)> = vec![("limit", &limit_str), ("depth", &depth_str)];

    let data: Vec<serde_json::Value> = client.get(&endpoint, &params).await?;
    let mut iter = data.into_iter();
    let post_value = iter
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid response from Reddit"))?;
    let comments_value = iter
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid response from Reddit"))?;

    let post_listing: Listing<RawPost> = serde_json::from_value(post_value)?;
    let comments_listing: Listing<RawComment> = serde_json::from_value(comments_value)?;

    let post: FormattedPost = post_listing
        .data
        .children
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Post not found"))?
        .data
        .into();

    let comments = build_comment_tree(&comments_listing.data.children, depth, limit);

    print_post_detail(&post, &comments);
    Ok(())
}
