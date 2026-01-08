use std::time::{SystemTime, UNIX_EPOCH};

/*
ENTRY is meant to mean somethign like, X(author) stated <body> at $TimeStamp
goal is to serve as a self contained immutable element of truth
*/

#[derive(Debug, Clone)]
pub struct Entry {
    author: String,
    body: String,
    timestamp: u64,
}

impl Entry {

    // Creating a new ENTRY
    pub fn new(author: impl Into<String>, body: impl Into<String>) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before unix epoch").as_secs();
        Self {
            author: author.into(),
            body: body.into(),
            timestamp: now,
        }
    }

    // author
    pub fn author(&self) -> &str {
        &self.author
    }

    // content body
    pub fn body(&self) -> &str {
        &self.body
    }

    // $TimeStamp
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}
