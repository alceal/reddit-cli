# Changelog

## [0.1.1] - 2026-03-15

### Security
- Complete terminal escape sanitization (CSI, OSC, DCS, 8-bit CSI sequences)
- Credential and token zeroization with `zeroize` crate
- Strict CLI argument validation with clap value parsers
- Response size limit (10MB) and HTTP timeout (10s)
- Redirect policy limited to 3 hops
- Error response body truncated to 1KB
- Input validation for subreddit names, usernames, and post IDs
- User-Agent header sanitization
- Custom Debug impl to mask secrets

### Added
- Shields.io badges in README
- Crate metadata for crates.io publishing
- `#[serde(default)]` on API response types for resilience

## [0.1.0] - 2026-03-14

### Added
- Initial release
- Browse subreddit posts with sort and time filters
- Search Reddit with subreddit filtering
- View post details with nested comment trees
- View user profiles with recent posts and comments
- View comments with sort options
- Colored terminal output
- OAuth2 authentication with token caching
