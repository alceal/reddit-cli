mod client;
mod commands;
mod format;
mod models;
mod validation;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "reddit-cli", version, about = "A CLI for browsing Reddit")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Browse posts from a subreddit
    Browse {
        /// Subreddit name (without r/ prefix)
        subreddit: String,
        /// Sort order: hot, new, top, rising, controversial
        #[arg(short, long, default_value = "hot", value_parser = clap::builder::PossibleValuesParser::new(["hot", "new", "top", "rising", "controversial"]))]
        sort: String,
        /// Number of posts to fetch (1-100)
        #[arg(short, long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..=100))]
        limit: u32,
        /// Time filter for top/controversial: hour, day, week, month, year, all
        #[arg(short, long, default_value = "day", value_parser = clap::builder::PossibleValuesParser::new(["hour", "day", "week", "month", "year", "all"]))]
        time: String,
    },
    /// Search Reddit
    Search {
        /// Search query
        query: String,
        /// Subreddit to search within
        #[arg(short = 'r', long)]
        subreddit: Option<String>,
        /// Sort order: relevance, hot, top, new, comments
        #[arg(short, long, default_value = "relevance", value_parser = clap::builder::PossibleValuesParser::new(["relevance", "hot", "top", "new", "comments"]))]
        sort: String,
        /// Number of results (1-100)
        #[arg(short, long, default_value_t = 25, value_parser = clap::value_parser!(u32).range(1..=100))]
        limit: u32,
        /// Time filter: hour, day, week, month, year, all
        #[arg(short, long, default_value = "all", value_parser = clap::builder::PossibleValuesParser::new(["hour", "day", "week", "month", "year", "all"]))]
        time: String,
    },
    /// Get post details with comments
    Post {
        /// Post ID or full Reddit URL
        post_id: String,
        /// Maximum depth of comment replies (0-10)
        #[arg(short, long, default_value_t = 3, value_parser = clap::value_parser!(u32).range(0..=10))]
        depth: u32,
        /// Maximum number of top-level comments (1-200)
        #[arg(short, long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..=200))]
        limit: u32,
    },
    /// Get user profile and activity
    User {
        /// Reddit username (without u/ prefix)
        username: String,
        /// Include recent posts
        #[arg(short, long)]
        posts: bool,
        /// Include recent comments
        #[arg(short, long)]
        comments: bool,
    },
    /// Submit a text post to a subreddit
    Submit {
        /// Subreddit to post to (without r/ prefix)
        subreddit: String,
        /// Post title
        #[arg(short, long)]
        title: String,
        /// Post body text
        #[arg(short, long, default_value = "")]
        body: String,
    },
    /// Get comments from a post
    Comments {
        /// Post ID or full Reddit URL
        post_id: String,
        /// Sort order: best, top, new, controversial, old, qa
        #[arg(short, long, default_value = "best", value_parser = clap::builder::PossibleValuesParser::new(["best", "top", "new", "controversial", "old", "qa"]))]
        sort: String,
        /// Maximum number of comments (1-200)
        #[arg(short, long, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..=200))]
        limit: u32,
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();

    let client = match client::RedditClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Browse {
            subreddit,
            sort,
            limit,
            time,
        } => commands::browse::execute(&client, &subreddit, &sort, limit, &time).await,
        Commands::Search {
            query,
            subreddit,
            sort,
            limit,
            time,
        } => {
            commands::search::execute(&client, &query, subreddit.as_deref(), &sort, limit, &time)
                .await
        }
        Commands::Post {
            post_id,
            depth,
            limit,
        } => commands::post::execute(&client, &post_id, depth, limit).await,
        Commands::User {
            username,
            posts,
            comments,
        } => commands::user::execute(&client, &username, posts, comments).await,
        Commands::Submit {
            subreddit,
            title,
            body,
        } => commands::submit::execute(&client, &subreddit, &title, &body).await,
        Commands::Comments {
            post_id,
            sort,
            limit,
        } => commands::comments::execute(&client, &post_id, &sort, limit).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
