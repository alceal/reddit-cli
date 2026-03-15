# reddit-cli

A command-line interface for browsing Reddit, built in Rust. Provides five commands for browsing subreddits, searching posts, reading post details with comment trees, viewing user profiles, and fetching comments.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.80+ for `LazyLock` support)
- A Reddit account
- Reddit API credentials (OAuth2 "script" app)

## Setup

### 1. Create a Reddit App

1. Go to [Reddit App Preferences](https://www.reddit.com/prefs/apps).
2. Click **"create another app..."** at the bottom.
3. Fill in:
   - **name:** `reddit-cli` (or any name)
   - **type:** select **script**
   - **redirect uri:** `http://localhost:8080`
4. Click **"create app"**.
5. Note the **client ID** (string under the app name) and the **client secret**.

> **Note:** Password-based OAuth does not work with accounts that have two-factor authentication (2FA) enabled. Use an account without 2FA or disable it temporarily.

### 2. Configure Credentials

```bash
cp .env.example .env
```

Edit `.env` with your credentials:

```
REDDIT_CLIENT_ID=your_client_id
REDDIT_CLIENT_SECRET=your_client_secret
REDDIT_USERNAME=your_reddit_username
REDDIT_PASSWORD=your_reddit_password
```

### 3. Build

```bash
cargo build --release
```

The binary will be at `target/release/reddit-cli`.

## Usage

### Browse a Subreddit

Fetch posts from a subreddit with a given sort order.

```bash
reddit-cli browse <subreddit> [OPTIONS]
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--sort` | `-s` | `hot` | Sort order: `hot`, `new`, `top`, `rising`, `controversial` |
| `--limit` | `-l` | `25` | Number of posts to fetch (1-100) |
| `--time` | `-t` | `day` | Time filter for `top`/`controversial`: `hour`, `day`, `week`, `month`, `year`, `all` |

**Examples:**

```bash
# Hot posts from r/rust
reddit-cli browse rust

# Top posts from r/programming this week
reddit-cli browse programming --sort top --time week --limit 10

# New posts from r/linux
reddit-cli browse linux --sort new --limit 5
```

### Search Reddit

Search across all of Reddit or within a specific subreddit.

```bash
reddit-cli search <query> [OPTIONS]
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--subreddit` | `-r` | | Restrict search to a subreddit |
| `--sort` | `-s` | `relevance` | Sort order: `relevance`, `hot`, `top`, `new`, `comments` |
| `--limit` | `-l` | `25` | Number of results (1-100) |
| `--time` | `-t` | `all` | Time filter: `hour`, `day`, `week`, `month`, `year`, `all` |

**Examples:**

```bash
# Search all of Reddit
reddit-cli search "async runtime"

# Search within a subreddit
reddit-cli search "error handling" --subreddit rust --sort top --limit 10

# Recent results only
reddit-cli search "release" --subreddit linux --time week
```

### View a Post

Get a post's full details including its comment tree.

```bash
reddit-cli post <post_id> [OPTIONS]
```

The `post_id` argument accepts:
- A bare post ID (e.g., `abc123`)
- A full Reddit URL (e.g., `https://www.reddit.com/r/rust/comments/abc123/...`)
- A short URL (e.g., `https://redd.it/abc123`)

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--depth` | `-d` | `3` | Maximum depth of nested comment replies (0-10) |
| `--limit` | `-l` | `50` | Maximum number of top-level comments (1-200) |

**Examples:**

```bash
# View a post by ID
reddit-cli post abc123

# View with deeper comment nesting
reddit-cli post abc123 --depth 5 --limit 20

# View from a full URL
reddit-cli post "https://www.reddit.com/r/rust/comments/abc123/some_title/"
```

### View a User Profile

Get a user's profile information with optional recent posts and comments.

```bash
reddit-cli user <username> [OPTIONS]
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--posts` | `-p` | `false` | Include the user's recent posts |
| `--comments` | `-c` | `false` | Include the user's recent comments |

**Examples:**

```bash
# Basic profile info
reddit-cli user spez

# Profile with recent posts and comments
reddit-cli user spez --posts --comments

# Just recent comments
reddit-cli user spez --comments
```

### View Comments

Fetch comments from a post with a specific sort order. Similar to `post` but focuses only on the comment tree.

```bash
reddit-cli comments <post_id> [OPTIONS]
```

| Option | Short | Default | Description |
|--------|-------|---------|-------------|
| `--sort` | `-s` | `best` | Sort order: `best`, `top`, `new`, `controversial`, `old`, `qa` |
| `--limit` | `-l` | `50` | Maximum number of comments (1-200) |

**Examples:**

```bash
# Best comments
reddit-cli comments abc123

# Top comments, limited to 10
reddit-cli comments abc123 --sort top --limit 10

# Newest comments from a URL
reddit-cli comments "https://www.reddit.com/r/rust/comments/abc123/title/" --sort new
```

## Output Format

Posts are displayed as a numbered list with metadata:

```
[1] Title of the post
    r/subreddit | u/author | 1,234 pts (95%) | 56 comments | 2h ago
    https://reddit.com/r/subreddit/comments/abc123
```

Comments are displayed as an indented tree:

```
u/author | 42 pts | 3h ago
Comment body text...

  u/reply_author | 10 pts | 2h ago
  Reply body text...
```

User profiles show account stats:

```
u/username
Account age: 5y | Link karma: 12,345 | Comment karma: 67,890
```

## Project Structure

```
src/
  main.rs          -- CLI definition with clap, subcommand dispatch
  client.rs        -- Reddit OAuth client with token caching
  models.rs        -- Serde types for Reddit API responses
  validation.rs    -- Input validation with compiled regexes
  format.rs        -- Colored terminal output formatting
  commands/
    browse.rs      -- Browse subreddit posts
    search.rs      -- Search Reddit
    post.rs        -- View post with comments
    user.rs        -- View user profile and activity
    comments.rs    -- View comments from a post
```

## License

MIT
