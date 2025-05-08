# Gorkit Bot

Gorkit is a Bluesky (AT Protocol) bot written in Rust. It monitors the network for specific mentions and automatically replies to them.

## Features

*   Listens to the AT Protocol's event stream (Jetstream) for new posts.
*   Specifically filters for posts on the `app.bsky.feed.post` collection.
*   Identifies posts that contain the exact text "@gork is this true".
*   Replies to these mentions with the text "yeh".
*   Persists its progress (cursor) to avoid reprocessing old messages on restart.
*   Includes basic setup for tracing (logging) and Prometheus metrics.

## Prerequisites

*   Rust toolchain installed.
*   A Bluesky account.

## Setup

1.  **Clone the repository (if applicable)**
    ```bash
    # git clone <your-repo-url>
    # cd gorkit
    ```

2.  **Set Environment Variables**:
    The bot requires your Bluesky credentials to log in and post replies. Create a `.env` file in the root of the project directory with the following content:

    ```env
    ATP_USER="your-bluesky-username"
    ATP_PASSWORD="your-bluesky-app-password"
    ```
    Replace `your-bluesky-username` with your Bluesky handle (e.g., `example.bsky.social`) and `your-bluesky-app-password` with an app password you generate from your Bluesky account settings. **Do not use your main account password.**

## Running the Bot

Once the environment variables are set up, you can run the bot using Cargo:

```bash
cargo run
```

The bot will start, log in to Bluesky, and begin listening for mentions.
