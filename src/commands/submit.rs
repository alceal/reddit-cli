use anyhow::{Result, bail};
use serde::Deserialize;

use crate::client::RedditClient;
use crate::validation::validate_subreddit;

#[derive(Deserialize)]
struct SubmitResponse {
    json: SubmitJson,
}

#[derive(Deserialize)]
struct SubmitJson {
    errors: Vec<Vec<String>>,
    data: Option<SubmitData>,
}

#[derive(Deserialize)]
struct SubmitData {
    url: String,
}

pub async fn execute(
    client: &RedditClient,
    subreddit: &str,
    title: &str,
    body: &str,
) -> Result<()> {
    let clean = validate_subreddit(subreddit)?;

    if title.is_empty() || title.len() > 300 {
        bail!("Title must be between 1 and 300 characters");
    }

    let form: Vec<(&str, &str)> = vec![
        ("sr", &clean),
        ("kind", "self"),
        ("title", title),
        ("text", body),
        ("api_type", "json"),
    ];

    let response: SubmitResponse = client.post_form("/api/submit", &form).await?;

    if !response.json.errors.is_empty() {
        let messages: Vec<String> = response.json.errors.iter().map(|e| e.join(": ")).collect();
        bail!("Reddit rejected the post: {}", messages.join("; "));
    }

    if let Some(data) = response.json.data {
        println!("Post submitted: {}", data.url);
    } else {
        println!("Post submitted successfully.");
    }

    Ok(())
}
