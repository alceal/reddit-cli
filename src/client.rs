use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use base64::Engine;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

use crate::models::{Listing, RawPost};

#[derive(Debug, thiserror::Error)]
pub enum RedditError {
    #[error("Reddit API error: {status} - {message}")]
    Api { status: u16, message: String },
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Rate limited by Reddit API")]
    RateLimited,
    #[error("Access forbidden")]
    Forbidden,
    #[error("Request timed out")]
    Timeout,
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

struct TokenCache {
    token: Option<zeroize::Zeroizing<String>>,
    expiry: Instant,
}

impl TokenCache {
    fn new() -> Self {
        Self {
            token: None,
            expiry: Instant::now(),
        }
    }
}

pub struct RedditClient {
    client: reqwest::Client,
    client_id: String,
    client_secret: Zeroizing<String>,
    username: String,
    password: Zeroizing<String>,
    token_cache: Mutex<TokenCache>,
}

impl std::fmt::Debug for RedditClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedditClient")
            .field("client_id", &self.client_id)
            .field("username", &self.username)
            .finish()
    }
}

impl RedditClient {
    pub fn new() -> Result<Self> {
        let client_id = std::env::var("REDDIT_CLIENT_ID").context("REDDIT_CLIENT_ID not set")?;
        let client_secret = Zeroizing::new(
            std::env::var("REDDIT_CLIENT_SECRET").context("REDDIT_CLIENT_SECRET not set")?,
        );
        let username = std::env::var("REDDIT_USERNAME").context("REDDIT_USERNAME not set")?;
        let password =
            Zeroizing::new(std::env::var("REDDIT_PASSWORD").context("REDDIT_PASSWORD not set")?);

        let sanitized_username = username.replace(['\n', '\r'], "");
        let user_agent = format!("reddit-cli/0.1.0 (by /u/{})", sanitized_username);
        let client = reqwest::Client::builder()
            .user_agent(&user_agent)
            .redirect(reqwest::redirect::Policy::limited(3))
            .build()?;

        Ok(Self {
            client,
            client_id,
            client_secret,
            username,
            password,
            token_cache: Mutex::new(TokenCache::new()),
        })
    }

    async fn get_token(&self) -> Result<zeroize::Zeroizing<String>> {
        let mut cache = self.token_cache.lock().await;

        if let Some(ref token) = cache.token
            && Instant::now() < cache.expiry
        {
            return Ok(token.clone());
        }

        let auth = base64::engine::general_purpose::STANDARD.encode(format!(
            "{}:{}",
            self.client_id,
            self.client_secret.as_str()
        ));

        let response = self
            .client
            .post("https://www.reddit.com/api/v1/access_token")
            .header("Authorization", format!("Basic {}", auth))
            .form(&[
                ("grant_type", "password"),
                ("username", self.username.as_str()),
                ("password", (*self.password).as_str()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(RedditError::Auth(format!("OAuth error: {}", response.status())).into());
        }

        let text = response.text().await?;
        let data: TokenResponse = serde_json::from_str(&text)
            .map_err(|_| RedditError::Auth("Failed to parse token response".into()))?;
        let token = zeroize::Zeroizing::new(data.access_token);
        cache.token = Some(token.clone());
        cache.expiry = Instant::now() + Duration::from_secs(data.expires_in.saturating_sub(60));

        Ok(token)
    }

    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let token = self.get_token().await?;
        let url = format!("https://oauth.reddit.com{}", endpoint);

        let mut query_params: Vec<(&str, &str)> = params.to_vec();
        query_params.push(("raw_json", "1"));

        let request = self
            .client
            .get(&url)
            .bearer_auth(token.as_str())
            .query(&query_params)
            .timeout(Duration::from_secs(10));

        let response = match request.send().await {
            Ok(r) => r,
            Err(e) if e.is_timeout() => return Err(RedditError::Timeout.into()),
            Err(e) => return Err(RedditError::Http(e).into()),
        };

        let status = response.status();
        if !status.is_success() {
            return match status.as_u16() {
                403 => Err(RedditError::Forbidden.into()),
                404 => Err(RedditError::NotFound("Resource not found".into()).into()),
                429 => Err(RedditError::RateLimited.into()),
                _ => {
                    let body = match response.bytes().await {
                        Ok(b) => String::from_utf8_lossy(&b[..b.len().min(1024)]).to_string(),
                        Err(_) => "(unable to read error body)".to_string(),
                    };
                    Err(RedditError::Api {
                        status: status.as_u16(),
                        message: body,
                    }
                    .into())
                }
            };
        }

        const MAX_RESPONSE_SIZE: u64 = 10_485_760;
        if let Some(content_length) = response.content_length()
            && content_length > MAX_RESPONSE_SIZE
        {
            return Err(anyhow::anyhow!(
                "Response too large: {} bytes (limit: {} bytes)",
                content_length,
                MAX_RESPONSE_SIZE
            ));
        }

        let text = response.text().await?;
        serde_json::from_str::<T>(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))
    }

    pub async fn post_form<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        form: &[(&str, &str)],
    ) -> Result<T> {
        let token = self.get_token().await?;
        let url = format!("https://oauth.reddit.com{}", endpoint);

        let request = self
            .client
            .post(&url)
            .bearer_auth(token.as_str())
            .form(form)
            .timeout(Duration::from_secs(10));

        let response = match request.send().await {
            Ok(r) => r,
            Err(e) if e.is_timeout() => return Err(RedditError::Timeout.into()),
            Err(e) => return Err(RedditError::Http(e).into()),
        };

        let status = response.status();
        if !status.is_success() {
            return match status.as_u16() {
                403 => Err(RedditError::Forbidden.into()),
                404 => Err(RedditError::NotFound("Resource not found".into()).into()),
                429 => Err(RedditError::RateLimited.into()),
                _ => {
                    let body = match response.bytes().await {
                        Ok(b) => String::from_utf8_lossy(&b[..b.len().min(1024)]).to_string(),
                        Err(_) => "(unable to read error body)".to_string(),
                    };
                    Err(RedditError::Api {
                        status: status.as_u16(),
                        message: body,
                    }
                    .into())
                }
            };
        }

        let text = response.text().await?;
        serde_json::from_str::<T>(&text)
            .map_err(|e| anyhow::anyhow!("Failed to parse response: {}", e))
    }

    pub async fn resolve_subreddit(&self, post_id: &str) -> Result<String> {
        let id_param = format!("t3_{}", post_id);
        let data: Listing<RawPost> = self.get("/api/info", &[("id", id_param.as_str())]).await?;

        let post = data
            .data
            .children
            .first()
            .ok_or_else(|| RedditError::NotFound("Post not found".into()))?;

        Ok(post.data.subreddit.clone())
    }
}
