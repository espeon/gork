use std::sync::{Arc, Mutex};

use cursor::load_cursor;
use jacquard::client::{Agent, MemoryCredentialSession};
use metrics_exporter_prometheus::PrometheusBuilder;
use tracing::{error, info};

use rocketman::{
    connection::JetstreamConnection,
    handler::{self, Ingestors},
    options::JetstreamOptions,
};

use crate::ingestors::app_bsky_feed_post::AppBskyFeedPostIngestor;

mod cursor;
mod ingestors;

fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

fn setup_metrics() {
    // Initialize metrics here
    if let Err(e) = PrometheusBuilder::new().install() {
        error!(
            "Failed to install, program will run without Prometheus exporter: {}",
            e
        );
    }
}

async fn setup_bsky_sess() -> anyhow::Result<Agent<MemoryCredentialSession>> {
    let (session, auth) = MemoryCredentialSession::authenticated(
        std::env::var("ATP_USER")?.into(),
        std::env::var("ATP_PASSWORD")?.into(),
        None,
        None,
    )
    .await?;
    let agent: Agent<_> = Agent::from(session);
    info!("logged in as {}", auth.handle);

    Ok(agent)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    setup_tracing();
    setup_metrics();
    info!("gorkin it...");

    let agent = Arc::new(match setup_bsky_sess().await {
        Ok(r) => r,
        Err(e) => panic!("{}", e.to_string()),
    });
    // init the builder
    let opts = JetstreamOptions::builder()
        // your EXACT nsids
        .wanted_collections(vec![
            "app.bsky.feed.post".to_string(),
            "place.stream.chat.message".to_string(),
        ])
        .bound(8 * 8 * 8 * 8 * 8 * 8) // 262144
        .build();
    // create the jetstream connector
    let jetstream = JetstreamConnection::new(opts);

    // create your ingestors
    let mut ingestors = Ingestors::new();

    ingestors.commits.insert(
        // your EXACT nsid
        "app.bsky.feed.post".to_string(),
        Box::new(AppBskyFeedPostIngestor::new(agent.clone())),
    );

    ingestors.commits.insert(
        // your EXACT nsid
        "place.stream.chat.message".to_string(),
        Box::new(ingestors::place_stream_chat_message::PlaceStreamChatMessageIngestor::new(agent)),
    );

    // arc it
    let ingestors = Arc::new(ingestors);

    let cursor: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(load_cursor().await));

    let msg_rx = jetstream.get_msg_rx();
    let reconnect_tx = jetstream.get_reconnect_tx();

    // spawn 10 tasks to process messages from the queue concurrently
    for i in 0..10 {
        let msg_rx_clone = msg_rx.clone();
        let ingestors_clone = Arc::clone(&ingestors);
        let reconnect_tx_clone = reconnect_tx.clone();
        let c_cursor = cursor.clone();

        tokio::spawn(async move {
            info!("Starting worker thread {}", i);
            while let Ok(message) = msg_rx_clone.recv_async().await {
                if let Err(e) = handler::handle_message(
                    message,
                    &ingestors_clone,
                    reconnect_tx_clone.clone(),
                    c_cursor.clone(),
                )
                .await
                {
                    eprintln!("Error processing message in worker {}: {}", i, e);
                };
            }
        });
    }

    let c_cursor = cursor.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let cursor_to_store: Option<u64> = {
                let cursor_guard = c_cursor.lock().unwrap();
                *cursor_guard
            };
            if let Some(cursor) = cursor_to_store {
                if let Err(e) = cursor::store_cursor(cursor).await {
                    error!("Error storing cursor: {}", e);
                }
            }
        }
    });

    // connect to jetstream
    // retries internally, but may fail if there is an extreme error.
    if let Err(e) = jetstream.connect(cursor.clone()).await {
        eprintln!("Failed to connect to Jetstream: {}", e);
        std::process::exit(1);
    }
}
