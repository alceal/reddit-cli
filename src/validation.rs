use std::sync::LazyLock;

use anyhow::{bail, Result};
use regex::Regex;

static SUBREDDIT_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]{1,21}$").unwrap());

static USERNAME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_\-]{3,20}$").unwrap());

static POST_ID_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]{1,10}$").unwrap());

static URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)reddit\.com/r/([^/]+)/comments/([a-z0-9]+)").unwrap()
});

static SHORT_URL_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)redd\.it/([a-z0-9]+)").unwrap());

pub fn validate_subreddit(name: &str) -> Result<String> {
    let cleaned = name.strip_prefix("r/").unwrap_or(name).trim();
    if cleaned.is_empty() || !SUBREDDIT_RE.is_match(cleaned) {
        bail!("Invalid subreddit name: {}", name);
    }
    Ok(cleaned.to_string())
}

pub fn validate_username(name: &str) -> Result<String> {
    let cleaned = name.strip_prefix("u/").unwrap_or(name).trim();
    if cleaned.is_empty() || !USERNAME_RE.is_match(cleaned) {
        bail!("Invalid username: {}", name);
    }
    Ok(cleaned.to_string())
}

pub fn validate_post_id(id: &str) -> Result<String> {
    let cleaned = id.trim();
    if cleaned.is_empty() || !POST_ID_RE.is_match(cleaned) {
        bail!("Invalid post ID: {}", id);
    }
    Ok(cleaned.to_string())
}

pub struct ExtractedPost {
    pub post_id: String,
    pub subreddit: Option<String>,
}

pub fn extract_post_id(input: &str) -> Result<ExtractedPost> {
    if let Some(caps) = URL_RE.captures(input) {
        let subreddit = validate_subreddit(&caps[1])?;
        let post_id = validate_post_id(&caps[2])?;
        return Ok(ExtractedPost {
            subreddit: Some(subreddit),
            post_id,
        });
    }

    if let Some(caps) = SHORT_URL_RE.captures(input) {
        let post_id = validate_post_id(&caps[1])?;
        return Ok(ExtractedPost {
            subreddit: None,
            post_id,
        });
    }

    let post_id = validate_post_id(input)?;
    Ok(ExtractedPost {
        subreddit: None,
        post_id,
    })
}
