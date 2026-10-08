use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use colored::Colorize;
use regex::Regex;

use crate::models::{FormattedComment, FormattedPost, FormattedUser, UserComment};

// Reddit embeds flair emoji in link_flair_text as `:name:` placeholders
// (e.g. "Doomsday :Doomsday:"). Requiring a leading letter keeps text such as
// "10:30:00" intact.
static FLAIR_EMOJI_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r":[A-Za-z][A-Za-z0-9_-]*:").unwrap());

fn sanitize(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some(&'[') => {
                    // CSI sequence: \x1b[ ... <letter>
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next.is_ascii_alphabetic() || next == '~' {
                            break;
                        }
                    }
                }
                Some(&']') => {
                    // OSC sequence: \x1b] ... terminated by BEL (\x07) or ST (\x1b\\)
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        if next == '\x07' {
                            chars.next();
                            break;
                        }
                        if next == '\x1b' {
                            chars.next();
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                        chars.next();
                    }
                }
                Some(&'P') => {
                    // DCS sequence: \x1bP ... terminated by ST (\x1b\\)
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        if next == '\x1b' {
                            chars.next();
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                        chars.next();
                    }
                }
                Some(_) => {
                    // Other escape: skip one character after \x1b
                    chars.next();
                }
                None => {}
            }
        } else if c == '\u{9b}' {
            // 8-bit CSI: same as \x1b[
            while let Some(&next) = chars.peek() {
                chars.next();
                if next.is_ascii_alphabetic() || next == '~' {
                    break;
                }
            }
        } else if c == '\n' || c == '\t' || !c.is_control() {
            result.push(c);
        }
    }
    result
}

pub fn format_time_ago(created_utc: f64) -> String {
    let Some(created) = DateTime::<Utc>::from_timestamp(created_utc as i64, 0) else {
        return "unknown".to_string();
    };
    let secs = (Utc::now() - created).num_seconds();

    if secs < 60 {
        format!("{}s ago", secs)
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else if secs < 2_592_000 {
        format!("{}d ago", secs / 86400)
    } else if secs < 31_536_000 {
        format!("{}mo ago", secs / 2_592_000)
    } else {
        format!("{}y ago", secs / 31_536_000)
    }
}

pub fn format_number(n: i64) -> String {
    let negative = n < 0;
    let s = n.unsigned_abs().to_string();
    let bytes = s.as_bytes();
    let mut result = String::new();
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(b as char);
    }
    if negative {
        format!("-{}", result)
    } else {
        result
    }
}

/// Flair text safe for a single output line: terminal escapes removed,
/// whitespace collapsed, and `:emoji:` placeholders dropped. An emoji-only
/// flair falls back to its placeholder text so the flair is not lost.
fn format_flair(raw: &str) -> Option<String> {
    let collapse = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
    let safe = sanitize(raw);
    let text = collapse(&FLAIR_EMOJI_RE.replace_all(&safe, " "));
    if !text.is_empty() {
        return Some(text);
    }
    let fallback = collapse(&safe);
    (!fallback.is_empty()).then_some(fallback)
}

/// The `r/sub | u/author | pts | comments | age` line, with `| [flair]`
/// appended when the post has one.
fn post_meta(post: &FormattedPost) -> String {
    let mut meta = format!(
        "{} | {} | {} pts ({}%) | {} comments | {}",
        format!("r/{}", sanitize(&post.subreddit)).cyan(),
        format!("u/{}", sanitize(&post.author)).yellow(),
        format_number(post.score).green(),
        (post.upvote_ratio * 100.0) as u32,
        format_number(post.num_comments),
        format_time_ago(post.created_utc).dimmed(),
    );
    if let Some(flair) = post.flair.as_deref().and_then(format_flair) {
        meta.push_str(&format!(" | {}", format!("[{}]", flair).magenta()));
    }
    meta
}

pub fn print_posts_list(posts: &[FormattedPost]) {
    if posts.is_empty() {
        println!("{}", "No posts found.".dimmed());
        return;
    }
    for (i, post) in posts.iter().enumerate() {
        println!(
            "{} {}",
            format!("[{}]", i + 1).dimmed(),
            sanitize(&post.title).bold()
        );
        println!("    {}", post_meta(post));
        println!("    {}", post.permalink.blue());
        if i < posts.len() - 1 {
            println!();
        }
    }
}

pub fn print_post_detail(post: &FormattedPost, comments: &[FormattedComment]) {
    println!("{}", sanitize(&post.title).bold());
    println!("{}", post_meta(post));
    println!("{}", post.permalink.blue());

    if let Some(ref text) = post.selftext {
        println!();
        println!("{}", sanitize(text));
    }

    if !comments.is_empty() {
        println!();
        println!("{}", "--- Comments ---".bold());
        println!();
        print_comment_tree(comments);
    }
}

pub fn print_comments(comments: &[FormattedComment]) {
    if comments.is_empty() {
        println!("{}", "No comments found.".dimmed());
        return;
    }
    print_comment_tree(comments);
}

fn print_comment_tree(comments: &[FormattedComment]) {
    for comment in comments {
        print_single_comment(comment);
    }
}

fn print_single_comment(comment: &FormattedComment) {
    let indent = "  ".repeat(comment.depth as usize);
    println!(
        "{}{} | {} | {}",
        indent,
        format!("u/{}", sanitize(&comment.author)).yellow(),
        format!("{} pts", format_number(comment.score)).green(),
        format_time_ago(comment.created_utc).dimmed(),
    );
    for line in comment.body.lines() {
        println!("{}{}", indent, sanitize(line));
    }
    println!();

    if let Some(ref replies) = comment.replies {
        for reply in replies {
            print_single_comment(reply);
        }
    }
}

pub fn print_user(
    user: &FormattedUser,
    posts: Option<&[FormattedPost]>,
    comments: Option<&[UserComment]>,
) {
    println!("{}", format!("u/{}", sanitize(&user.name)).bold());
    println!(
        "Account age: {} | Link karma: {} | Comment karma: {}",
        format_time_ago(user.created_utc),
        format_number(user.link_karma).green(),
        format_number(user.comment_karma).green(),
    );

    if let Some(posts) = posts {
        println!();
        println!("{}", "--- Recent Posts ---".bold());
        println!();
        print_posts_list(posts);
    }

    if let Some(comments) = comments {
        println!();
        println!("{}", "--- Recent Comments ---".bold());
        println!();
        for comment in comments {
            println!(
                "in {} | {} | {}",
                format!("r/{}", sanitize(&comment.subreddit)).cyan(),
                format!("{} pts", format_number(comment.score)).green(),
                format_time_ago(comment.created_utc).dimmed(),
            );
            println!("  Re: {}", sanitize(&comment.link_title).dimmed());
            for line in comment.body.lines() {
                println!("  {}", sanitize(line));
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn post_with_flair(flair: Option<&str>) -> FormattedPost {
        FormattedPost {
            id: "abc123".into(),
            title: "t".into(),
            author: "alice".into(),
            subreddit: "rust".into(),
            score: 1234,
            upvote_ratio: 0.95,
            num_comments: 56,
            created_utc: Utc::now().timestamp() as f64 - 7200.0,
            url: String::new(),
            selftext: None,
            is_self: true,
            permalink: "https://reddit.com/r/rust/comments/abc123/t/".into(),
            flair: flair.map(str::to_string),
        }
    }

    #[test]
    fn plain_flair_is_kept() {
        assert_eq!(format_flair("Meme").as_deref(), Some("Meme"));
        assert_eq!(
            format_flair("Built with Claude").as_deref(),
            Some("Built with Claude")
        );
    }

    #[test]
    fn emoji_placeholders_are_removed() {
        assert_eq!(
            format_flair("Doomsday :Doomsday:").as_deref(),
            Some("Doomsday")
        );
        assert_eq!(
            format_flair(":redditgold: Workaround").as_deref(),
            Some("Workaround")
        );
        assert_eq!(
            format_flair("Avengers :Post_Avengers:").as_deref(),
            Some("Avengers")
        );
    }

    #[test]
    fn emoji_only_flair_falls_back_to_placeholder() {
        assert_eq!(format_flair(":Doomsday:").as_deref(), Some(":Doomsday:"));
    }

    #[test]
    fn colons_in_numbers_are_not_emoji() {
        assert_eq!(
            format_flair("Live 10:30:00").as_deref(),
            Some("Live 10:30:00")
        );
    }

    #[test]
    fn flair_is_sanitized_and_single_line() {
        assert_eq!(
            format_flair("Me\x1b[31mme\nPart\t2").as_deref(),
            Some("Meme Part 2")
        );
        assert_eq!(format_flair("   "), None);
        assert_eq!(format_flair("\x1b[2J"), None);
    }

    #[test]
    fn meta_line_appends_flair_only_when_present() {
        colored::control::set_override(false);
        assert_eq!(
            post_meta(&post_with_flair(Some("Doomsday :Doomsday:"))),
            "r/rust | u/alice | 1,234 pts (95%) | 56 comments | 2h ago | [Doomsday]"
        );
        assert_eq!(
            post_meta(&post_with_flair(None)),
            "r/rust | u/alice | 1,234 pts (95%) | 56 comments | 2h ago"
        );
    }
}
