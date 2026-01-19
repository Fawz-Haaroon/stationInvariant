use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct Fanout {
    streams: Arc<RwLock<HashMap<u64, broadcast::Sender<(u64, Vec<u8>)>>>>,
}

impl Fanout {
    pub fn new() -> Self {
        Self {
            streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(
        &self,
        stream_id: u64,
    ) -> broadcast::Receiver<(u64, Vec<u8>)> {
        let mut streams = self.streams.write().await;

        let sender = streams
            .entry(stream_id)
            .or_insert_with(|| {
                let (tx, _rx) = broadcast::channel(1024);
                tx
            });

        sender.subscribe()
    }

    pub async fn broadcast(
        &self,
        stream_id: u64,
        offset: u64,
        payload: Vec<u8>,
    ) {
        let streams = self.streams.read().await;

        if let Some(sender) = streams.get(&stream_id) {
            // Ignore error: it only means no active receivers
            let _ = sender.send((offset, payload));
        }
    }
}

