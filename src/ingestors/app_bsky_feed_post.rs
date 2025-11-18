use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use jacquard::{
    api::{
        app_bsky::{self, feed::post::Post},
        com_atproto::repo::strong_ref::StrongRef,
    },
    client::{Agent, AgentSessionExt, MemoryCredentialSession},
    types::{aturi::AtUri, string::Datetime, value},
};
use rocketman::{
    ingestion::LexiconIngestor,
    types::event::{Commit, Event},
};
use serde_json::Value;

use crate::ingestors::is_gork_mention;

pub struct AppBskyFeedPostIngestor {
    agent: Arc<Agent<MemoryCredentialSession>>,
}

impl AppBskyFeedPostIngestor {
    pub fn new(agent: Arc<Agent<MemoryCredentialSession>>) -> Self {
        Self { agent }
    }
}

/// A cool ingestor implementation.
#[async_trait]
impl LexiconIngestor for AppBskyFeedPostIngestor {
    async fn ingest(&self, message: Event<Value>) -> anyhow::Result<()> {
        if let Some(Commit {
            record: Some(record),
            cid: Some(cid),
            rkey,
            collection,
            operation: _,
            rev: _,
        }) = message.commit
        {
            let poast: app_bsky::feed::post::Post =
                value::from_json_value::<app_bsky::feed::post::Post>(record.clone())?;

            if !is_gork_mention(&poast.text) {
                return Ok(());
            };
            // get the strongref to da post
            let rcid = StrongRef::new()
                .cid(cid)
                .uri(AtUri::from_str(&format!(
                    "at://{}/{}/{}",
                    message.did, collection, rkey
                ))?)
                .build();

            // get parent CID of above post, else get above post's CID
            let parent_cid = match poast.reply {
                Some(reply) => reply.parent,
                None => rcid.clone(),
            };

            let post = Post::new()
                .reply(app_bsky::feed::post::ReplyRef {
                    parent: parent_cid,
                    root: rcid,
                    extra_data: None,
                })
                .text("yeh")
                .created_at(Datetime::now())
                .build();

            self.agent.create_record(post, None).await?;
        }
        Ok(())
    }
}
