pub mod app_bsky_feed_post;
pub mod place_stream_chat_message;

pub fn is_gork_mention(text: &str) -> bool {
    (text.starts_with("@gork.bluesky.bot") || text.starts_with("@gork.it"))
        && (text.contains("is this") || text.contains("am i") || text.contains("do you"))
}
