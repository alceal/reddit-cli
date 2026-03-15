use anyhow::Result;

use crate::client::RedditClient;
use crate::format::print_user;
use crate::models::{
    FormattedPost, FormattedUser, Listing, RawPost, RawUser, RawUserComment, Thing, UserComment,
};
use crate::validation::validate_username;

pub async fn execute(
    client: &RedditClient,
    username: &str,
    include_posts: bool,
    include_comments: bool,
) -> Result<()> {
    let clean = validate_username(username)?;

    let about_endpoint = format!("/user/{}/about", clean);
    let about: Thing<RawUser> = client.get(&about_endpoint, &[]).await?;
    let user: FormattedUser = about.data.into();

    let posts = if include_posts {
        let endpoint = format!("/user/{}/submitted", clean);
        let data: Listing<RawPost> = client.get(&endpoint, &[("limit", "25")]).await?;
        let posts: Vec<FormattedPost> = data
            .data
            .children
            .into_iter()
            .map(|t| t.data.into())
            .collect();
        Some(posts)
    } else {
        None
    };

    let comments = if include_comments {
        let endpoint = format!("/user/{}/comments", clean);
        let data: Listing<RawUserComment> = client.get(&endpoint, &[("limit", "25")]).await?;
        let comments: Vec<UserComment> = data
            .data
            .children
            .into_iter()
            .map(|t| t.data.into())
            .collect();
        Some(comments)
    } else {
        None
    };

    print_user(
        &user,
        posts.as_deref(),
        comments.as_deref(),
    );
    Ok(())
}
