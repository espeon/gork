use std::{str::FromStr, sync::Arc};

use async_trait::async_trait;
use jacquard::{
    api::{com_atproto::repo::strong_ref::StrongRef, place_stream},
    client::{Agent, AgentSessionExt, MemoryCredentialSession},
    types::{aturi::AtUri, string::Datetime, value},
};
use rocketman::{
    ingestion::LexiconIngestor,
    types::event::{Commit, Event},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ingestors::is_gork_mention;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceStreamChatMessage {
    pub text: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embed: Option<Value>,
}

pub struct PlaceStreamChatMessageIngestor {
    agent: Arc<Agent<MemoryCredentialSession>>,
}

impl PlaceStreamChatMessageIngestor {
    pub fn new(agent: Arc<Agent<MemoryCredentialSession>>) -> Self {
        Self { agent }
    }
}

#[async_trait]
impl LexiconIngestor for PlaceStreamChatMessageIngestor {
    async fn ingest(&self, message: Event<Value>) -> anyhow::Result<()> {
        if let Some(Commit {
            record: Some(record),
            cid: Some(cid),
            rkey,
            collection: _,
            operation: _,
            rev: _,
        }) = message.commit
        {
            let chat_message =
                value::from_json_value::<place_stream::chat::message::Message>(record.clone())?;

            // check if message mentions gork
            if !is_gork_mention(&chat_message.text) {
                return Ok(());
            }

            let reply_strongref = StrongRef::new()
                .cid(cid)
                .uri(AtUri::from_str(&format!(
                    "at://{}/place.stream.chat.message/{}",
                    message.did, rkey
                ))?)
                .build();

            // build reply strongref
            let replyref = place_stream::chat::message::ReplyRef::new()
                .root(reply_strongref.clone())
                .parent(reply_strongref.clone())
                .build();

            let message = place_stream::chat::message::Message::new()
                .text("yeh")
                .reply(replyref)
                .created_at(Datetime::now())
                .streamer(chat_message.streamer)
                .build();

            self.agent.create_record(message, None).await?;
        }
        Ok(())
    }
}
