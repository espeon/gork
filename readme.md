## Gorkit Bot

Gorkit is a Bluesky (AT Protocol) bot written in Rust. Monitors specific posts and replies automatically.

### Features

* Subscribes to the AT Protocol event stream (Jetstream).
* Filters for posts in `app.bsky.feed.post`.
* Detects exact match: `@gork.bluesky.bot is this true`.
* Replies with `yeh`.
* Saves cursor to avoid duplicate processing on restart.
* Includes basic setup for tracing and Prometheus metrics.

### Requirements

* Rust toolchain installed.
* Bluesky account.

### Setup

1. **Clone the repository (if needed)**

   ```bash
   git clone <your-repo-url>
   cd gorkit
   ```

2. **Set environment variables**

   Create a `.env` file in the project root:

   ```env
   ATP_USER="your-bluesky-username"
   ATP_PASSWORD="your-bluesky-app-password"
   ```

   Replace values accordingly.
   Use an app password, not your main account password.

### Run

```bash
cargo run
```

Starts the bot, logs in, and listens for mentions.
