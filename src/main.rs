use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};

use atrium_api::{
    app::bsky::feed::post::ReplyRefData,
    com::atproto::repo::strong_ref::MainData,
    types::string::{Cid, Datetime},
};
use bsky_sdk::BskyAgent;
use cursor::load_cursor;
use metrics_exporter_prometheus::PrometheusBuilder;
use serde_json::Value;
use tracing::{error, info};

use rocketman::{
    connection::JetstreamConnection,
    handler,
    ingestion::LexiconIngestor,
    options::JetstreamOptions,
    types::event::{Commit, Event},
};

use async_trait::async_trait;

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

async fn setup_bsky_sess() -> anyhow::Result<BskyAgent> {
    let agent = BskyAgent::builder().client(client).build().await?;
    let res = agent
        .login(std::env::var("ATP_USER")?, std::env::var("ATP_PASSWORD")?)
        .await?;

    info!("logged in as {}", res.handle.to_string());

    Ok(agent)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    setup_tracing();
    setup_metrics();
    info!("gorkin it...");

    let agent = match setup_bsky_sess().await {
        Ok(r) => r,
        Err(e) => panic!("{}", e.to_string()),
    };
    // init the builder
    let opts = JetstreamOptions::builder()
        // your EXACT nsids
        .wanted_collections(vec!["app.bsky.feed.post".to_string()])
        .build();
    // create the jetstream connector
    let jetstream = JetstreamConnection::new(opts);

    // create your ingestors
    let mut ingestors: HashMap<String, Box<dyn LexiconIngestor + Send + Sync>> = HashMap::new();
    ingestors.insert(
        // your EXACT nsid
        "app.bsky.feed.post".to_string(),
        Box::new(MyCoolIngestor::new(agent.clone())),
    );

    // tracks the last message we've processed
    let cursor: Arc<Mutex<Option<u64>>> = Arc::new(Mutex::new(load_cursor().await));

    // get channels
    let msg_rx = jetstream.get_msg_rx();
    let reconnect_tx = jetstream.get_reconnect_tx();

    // spawn a task to process messages from the queue.
    // this is a simple implementation, you can use a more complex one based on needs.
    let c_cursor = cursor.clone();
    tokio::spawn(async move {
        while let Ok(message) = msg_rx.recv_async().await {
            if let Err(e) =
                handler::handle_message(message, &ingestors, reconnect_tx.clone(), c_cursor.clone())
                    .await
            {
                eprintln!("Error processing message: {}", e);
            };
        }
    });

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

pub struct MyCoolIngestor {
    agent: BskyAgent,
}

impl MyCoolIngestor {
    pub fn new(agent: BskyAgent) -> Self {
        Self { agent }
    }
}

/// A cool ingestor implementation.
#[async_trait]
impl LexiconIngestor for MyCoolIngestor {
    async fn ingest(&self, message: Event<Value>) -> anyhow::Result<()> {
        if let Some(Commit {
            record: Some(record),
            cid: Some(cid),
            rkey,
            collection,
            ..
        }) = message.commit
        {
            let riposte =
                serde_json::from_value::<atrium_api::app::bsky::feed::post::RecordData>(record)?;

            // if we good
            if riposte.text != "@gork is this true" {
                return Ok(());
            };
            // set the proper reply stuff to reply to mentioned post

            // get the cid
            let rcid = match Cid::from_str(&cid) {
                Ok(r) => r,
                Err(e) => return Err(anyhow::anyhow!(e)),
            };

            let reply = if let Some(mut reply) = riposte.reply {
                reply.parent = MainData {
                    cid: rcid,
                    uri: format!("at://{}/{}/{}", message.did, collection, rkey),
                }
                .into();
                Some(reply)
            } else {
                Some(
                    ReplyRefData {
                        parent: MainData {
                            cid: rcid.clone(),
                            uri: format!("at://{}/{}/{}", message.did, collection, rkey),
                        }
                        .into(),
                        root: MainData {
                            cid: rcid,
                            uri: format!("at://{}/{}/{}", message.did, collection, rkey),
                        }
                        .into(),
                    }
                    .into(),
                )
            };

            self.agent
                .create_record(atrium_api::app::bsky::feed::post::RecordData {
                    created_at: Datetime::now(),
                    embed: None,
                    entities: None,
                    facets: None,
                    labels: None,
                    langs: None,
                    reply,
                    tags: None,
                    text: "yeh".to_string(),
                })
                .await?;
        }
        Ok(())
    }
}
