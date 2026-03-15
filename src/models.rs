use serde::Deserialize;

// Reddit API listing wrapper

#[derive(Debug, Deserialize)]
pub struct Listing<T> {
    pub data: ListingData<T>,
}

#[derive(Debug, Deserialize)]
pub struct ListingData<T> {
    pub children: Vec<Thing<T>>,
}

#[derive(Debug, Deserialize)]
pub struct Thing<T> {
    pub kind: String,
    pub data: T,
}

// Raw API response types

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct RawPost {
    pub id: String,
    pub title: String,
    pub author: String,
    pub subreddit: String,
    pub score: i64,
    pub upvote_ratio: f64,
    pub num_comments: i64,
    pub created_utc: f64,
    pub url: String,
    pub selftext: Option<String>,
    pub is_self: bool,
    pub permalink: String,
}

impl Default for RawPost {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            author: String::new(),
            subreddit: String::new(),
            score: 0,
            upvote_ratio: 0.0,
            num_comments: 0,
            created_utc: 0.0,
            url: String::new(),
            selftext: None,
            is_self: false,
            permalink: String::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RawComment {
    pub id: Option<String>,
    pub author: Option<String>,
    pub body: Option<String>,
    pub score: Option<i64>,
    pub created_utc: Option<f64>,
    #[serde(default, deserialize_with = "deserialize_replies")]
    pub replies: Option<Listing<RawComment>>,
}

#[derive(Debug, Deserialize)]
pub struct RawUser {
    pub name: String,
    pub created_utc: f64,
    pub link_karma: i64,
    pub comment_karma: i64,
    #[serde(default)]
    pub is_gold: bool,
    #[serde(default)]
    pub is_mod: bool,
    #[serde(default)]
    pub verified: bool,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct RawUserComment {
    pub id: String,
    pub body: String,
    pub score: i64,
    pub subreddit: String,
    pub created_utc: f64,
    pub link_title: String,
}

impl Default for RawUserComment {
    fn default() -> Self {
        Self {
            id: String::new(),
            body: String::new(),
            score: 0,
            subreddit: String::new(),
            created_utc: 0.0,
            link_title: String::new(),
        }
    }
}

// Formatted output types

#[derive(Debug)]
#[allow(dead_code)]
pub struct FormattedPost {
    pub id: String,
    pub title: String,
    pub author: String,
    pub subreddit: String,
    pub score: i64,
    pub upvote_ratio: f64,
    pub num_comments: i64,
    pub created_utc: f64,
    pub url: String,
    pub selftext: Option<String>,
    pub is_self: bool,
    pub permalink: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FormattedComment {
    pub id: String,
    pub author: String,
    pub body: String,
    pub score: i64,
    pub created_utc: f64,
    pub depth: u32,
    pub replies: Option<Vec<FormattedComment>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct FormattedUser {
    pub name: String,
    pub created_utc: f64,
    pub link_karma: i64,
    pub comment_karma: i64,
    pub is_gold: bool,
    pub is_mod: bool,
    pub verified: bool,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct UserComment {
    pub id: String,
    pub body: String,
    pub score: i64,
    pub subreddit: String,
    pub created_utc: f64,
    pub link_title: String,
}

// Conversions

impl From<RawPost> for FormattedPost {
    fn from(raw: RawPost) -> Self {
        Self {
            id: raw.id,
            title: raw.title,
            author: raw.author,
            subreddit: raw.subreddit,
            score: raw.score,
            upvote_ratio: raw.upvote_ratio,
            num_comments: raw.num_comments,
            created_utc: raw.created_utc,
            url: raw.url,
            selftext: raw.selftext.filter(|s| !s.is_empty()),
            is_self: raw.is_self,
            permalink: format!("https://reddit.com{}", raw.permalink),
        }
    }
}

impl From<RawUser> for FormattedUser {
    fn from(raw: RawUser) -> Self {
        Self {
            name: raw.name,
            created_utc: raw.created_utc,
            link_karma: raw.link_karma,
            comment_karma: raw.comment_karma,
            is_gold: raw.is_gold,
            is_mod: raw.is_mod,
            verified: raw.verified,
        }
    }
}

impl From<RawUserComment> for UserComment {
    fn from(raw: RawUserComment) -> Self {
        Self {
            id: raw.id,
            body: raw.body,
            score: raw.score,
            subreddit: raw.subreddit,
            created_utc: raw.created_utc,
            link_title: raw.link_title,
        }
    }
}

// Comment tree building

pub fn build_comment_tree(
    children: &[Thing<RawComment>],
    max_depth: u32,
    limit: u32,
) -> Vec<FormattedComment> {
    children
        .iter()
        .filter(|t| t.kind == "t1")
        .take(limit as usize)
        .filter_map(|t| format_comment(&t.data, 0, max_depth))
        .collect()
}

fn format_comment(raw: &RawComment, depth: u32, max_depth: u32) -> Option<FormattedComment> {
    let author = raw.author.as_ref()?;

    let mut comment = FormattedComment {
        id: raw.id.clone().unwrap_or_default(),
        author: author.clone(),
        body: raw.body.clone().unwrap_or_default(),
        score: raw.score.unwrap_or(0),
        created_utc: raw.created_utc.unwrap_or(0.0),
        depth,
        replies: None,
    };

    if depth < max_depth
        && let Some(ref replies_listing) = raw.replies
    {
        let replies: Vec<FormattedComment> = replies_listing
            .data
            .children
            .iter()
            .filter(|t| t.kind == "t1")
            .filter_map(|t| format_comment(&t.data, depth + 1, max_depth))
            .collect();
        if !replies.is_empty() {
            comment.replies = Some(replies);
        }
    }

    Some(comment)
}

// Custom deserializer for Reddit's replies field (either "" or a nested Listing)

fn deserialize_replies<'de, D>(deserializer: D) -> Result<Option<Listing<RawComment>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_string() {
        Ok(None)
    } else if value.is_object() {
        serde_json::from_value(value)
            .map(Some)
            .map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}
