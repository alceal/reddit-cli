use chrono::{DateTime, Utc};
use colored::Colorize;

use crate::models::{FormattedComment, FormattedPost, FormattedUser, UserComment};

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
        println!(
            "    {} | {} | {} pts ({}%) | {} comments | {}",
            format!("r/{}", sanitize(&post.subreddit)).cyan(),
            format!("u/{}", sanitize(&post.author)).yellow(),
            format_number(post.score).green(),
            (post.upvote_ratio * 100.0) as u32,
            format_number(post.num_comments),
            format_time_ago(post.created_utc).dimmed(),
        );
        println!("    {}", post.permalink.blue());
        if i < posts.len() - 1 {
            println!();
        }
    }
}

pub fn print_post_detail(post: &FormattedPost, comments: &[FormattedComment]) {
    println!("{}", sanitize(&post.title).bold());
    println!(
        "{} | {} | {} pts ({}%) | {} comments | {}",
        format!("r/{}", sanitize(&post.subreddit)).cyan(),
        format!("u/{}", sanitize(&post.author)).yellow(),
        format_number(post.score).green(),
        (post.upvote_ratio * 100.0) as u32,
        format_number(post.num_comments),
        format_time_ago(post.created_utc).dimmed(),
    );
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
