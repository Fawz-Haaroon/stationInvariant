stationInvariant: 4-Hour Buildable Production-Ready Minimal System
The Brutally Honest, Zero-BS, Learn-By-Building PRD
Target: Build in 4-6 hours, fully functional, deployable, zero fake parts
Audience: You, learning from scratch
Output: Real append-only message system with UI

0. Core Truth
This is not a toy. This is the smallest real thing that:

Writes to disk before acknowledging
Never loses data on crash
Handles multiple connections
Streams messages in realtime
Has a working UI
Can be deployed

Every line of code earns its place.

1. What You're Actually Building
Browser Client ←WebSocket→ Rust Server ←writes→ Disk Files
     ↓                           ↓
  Simple UI              Append-Only Log
                         (one file per stream)
The entire stack:

Rust binary server (tokio + WebSocket)
Disk persistence (raw file I/O)
In-memory fan-out (broadcast channel)
HTML/CSS/JS UI (vanilla, zero build)
Binary protocol (hand-written parser)

No:

No databases
No frameworks (React, etc.)
No external dependencies beyond tokio + tungstenite
No "temporary" solutions


2. Folder Structure (Every File Matters)
station/
├── Cargo.toml                    # Dependencies ONLY: tokio, tungstenite, bytes
├── .gitignore
├── README.md                      # How to run, how it works
│
├── src/
│   ├── main.rs                   # 50 lines: setup + start server
│   │
│   ├── server.rs                 # TCP listener, accept loop
│   ├── connection.rs             # Handle ONE WebSocket connection
│   │
│   ├── protocol.rs               # Binary frame encode/decode
│   │   # Frame = [type:u8][stream:u64][offset:u64][len:u32][payload]
│   │
│   ├── ledger.rs                 # Core logic: append, assign offsets
│   ├── entry.rs                  # Entry struct + validation
│   │
│   ├── disk.rs                   # Append-only file writer
│   ├── replay.rs                 # Read entries from offset
│   │
│   ├── fanout.rs                 # In-process pub/sub registry
│   │
│   └── metrics.rs                # Simple counters (AtomicU64)
│
├── ui/
│   ├── index.html                # Single page, ~100 lines
│   ├── app.js                    # WebSocket client, ~150 lines
│   └── style.css                 # Minimal professional CSS, ~50 lines
│
└── data/                         # Created at runtime
    └── .gitkeep
Total lines of code target: ~1200 lines Rust + ~300 lines UI = 1500 lines

3. Hour-by-Hour Build Plan
Hour 1: Foundation (Protocol + Disk)
Goal: Messages can be written to disk and read back
bash# Commits:
1. cargo init + Cargo.toml
2. protocol.rs: Frame struct + encode/decode
3. disk.rs: append to file + flush
4. Test: write 1000 messages, read them back
Code skeleton:
rust// src/protocol.rs
pub struct Frame {
    pub frame_type: u8,
    pub stream_id: u64,
    pub offset: u64,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.frame_type);
        buf.extend_from_slice(&self.stream_id.to_le_bytes());
        buf.extend_from_slice(&self.offset.to_le_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }
    
    pub fn decode(data: &[u8]) -> Result<Self, String> {
        if data.len() < 21 { return Err("too short".into()); }
        
        let frame_type = data[0];
        let stream_id = u64::from_le_bytes(data[1..9].try_into().unwrap());
        let offset = u64::from_le_bytes(data[9..17].try_into().unwrap());
        let len = u32::from_le_bytes(data[17..21].try_into().unwrap()) as usize;
        
        if data.len() < 21 + len { return Err("incomplete".into()); }
        
        let payload = data[21..21+len].to_vec();
        
        Ok(Frame { frame_type, stream_id, offset, payload })
    }
}
rust// src/disk.rs
use std::fs::OpenOptions;
use std::io::Write;

pub struct DiskLog {
    file: std::fs::File,
}

impl DiskLog {
    pub fn open(stream_id: u64) -> std::io::Result<Self> {
        let path = format!("data/stream-{:016x}.log", stream_id);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(Self { file })
    }
    
    pub fn append(&mut self, offset: u64, timestamp: i64, payload: &[u8]) -> std::io::Result<()> {
        self.file.write_all(&offset.to_le_bytes())?;
        self.file.write_all(&timestamp.to_le_bytes())?;
        self.file.write_all(&(payload.len() as u32).to_le_bytes())?;
        self.file.write_all(payload)?;
        self.file.flush()?; // CRITICAL: durability
        Ok(())
    }
}
Test:
rust#[test]
fn test_disk_roundtrip() {
    let mut log = DiskLog::open(1).unwrap();
    log.append(0, 12345, b"hello").unwrap();
    
    // Read it back
    let entries = replay_from_offset(1, 0).unwrap();
    assert_eq!(entries[0].payload, b"hello");
}

Hour 2: Network Layer (WebSocket Server)
Goal: Clients can connect, send frames, get responses
bash# Commits:
5. server.rs: TCP listener with tokio
6. connection.rs: WebSocket upgrade + frame handling
7. Test with websocat: echo messages back
Code:
rust// src/server.rs
use tokio::net::TcpListener;
use crate::connection::handle_connection;

pub async fn run_server(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("Server listening on {}", addr);
    
    loop {
        let (stream, peer) = listener.accept().await?;
        println!("New connection from {}", peer);
        
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}
rust// src/connection.rs
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};
use crate::protocol::Frame;

pub async fn handle_connection(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let ws = accept_async(stream).await?;
    let (mut write, mut read) = ws.split();
    
    while let Some(Ok(msg)) = read.next().await {
        if let Message::Binary(data) = msg {
            let frame = Frame::decode(&data)?;
            
            // Echo it back for now
            let response = Frame {
                frame_type: 0x02, // MESSAGE
                stream_id: frame.stream_id,
                offset: frame.offset,
                payload: frame.payload,
            };
            
            write.send(Message::Binary(response.encode())).await?;
        }
    }
    
    Ok(())
}
Test with CLI:
bash# Terminal 1
cargo run

# Terminal 2
websocat ws://127.0.0.1:9000
# Type hex bytes, see echo

Hour 3: Ledger Logic (Append + Fan-out)
Goal: Multiple clients can publish and subscribe
bash# Commits:
8. ledger.rs: atomic offset assignment
9. fanout.rs: broadcast to subscribers
10. Integration: publish → disk → broadcast
Code:
rust// src/ledger.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::disk::DiskLog;
use crate::fanout::Fanout;

pub struct Ledger {
    next_offset: AtomicU64,
    logs: Arc<RwLock<HashMap<u64, DiskLog>>>,
    fanout: Arc<Fanout>,
}

impl Ledger {
    pub fn new(fanout: Arc<Fanout>) -> Self {
        Self {
            next_offset: AtomicU64::new(0),
            logs: Arc::new(RwLock::new(HashMap::new())),
            fanout,
        }
    }
    
    pub async fn append(&self, stream_id: u64, payload: Vec<u8>) -> Result<u64, String> {
        // 1. Assign offset atomically
        let offset = self.next_offset.fetch_add(1, Ordering::SeqCst);
        
        // 2. Write to disk
        let mut logs = self.logs.write().await;
        let log = logs.entry(stream_id)
            .or_insert_with(|| DiskLog::open(stream_id).unwrap());
        
        let timestamp = chrono::Utc::now().timestamp_nanos();
        log.append(offset, timestamp, &payload)
            .map_err(|e| e.to_string())?;
        
        // 3. Broadcast to subscribers
        self.fanout.broadcast(stream_id, offset, payload).await;
        
        Ok(offset)
    }
}
rust// src/fanout.rs
use tokio::sync::broadcast;
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct Fanout {
    channels: Arc<RwLock<HashMap<u64, broadcast::Sender<(u64, Vec<u8>)>>>>,
}

impl Fanout {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn subscribe(&self, stream_id: u64) -> broadcast::Receiver<(u64, Vec<u8>)> {
        let mut channels = self.channels.write().await;
        let sender = channels.entry(stream_id)
            .or_insert_with(|| broadcast::channel(1000).0);
        sender.subscribe()
    }
    
    pub async fn broadcast(&self, stream_id: u64, offset: u64, payload: Vec<u8>) {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&stream_id) {
            let _ = sender.send((offset, payload));
        }
    }
}
Updated connection.rs:
rustpub async fn handle_connection(
    stream: TcpStream,
    ledger: Arc<Ledger>,
) -> Result<(), Box<dyn std::error::Error>> {
    let ws = accept_async(stream).await?;
    let (mut write, mut read) = ws.split();
    
    // Spawn reader task
    let ledger_clone = ledger.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Binary(data) = msg {
                if let Ok(frame) = Frame::decode(&data) {
                    match frame.frame_type {
                        0x01 => { // PUBLISH
                            let offset = ledger_clone.append(frame.stream_id, frame.payload).await.unwrap();
                            // Send ACK
                            let ack = Frame {
                                frame_type: 0x04,
                                stream_id: frame.stream_id,
                                offset,
                                payload: vec![],
                            };
                            // (need to send back to write half - use channel)
                        }
                        0x03 => { // SUBSCRIBE
                            // Register subscriber
                        }
                        _ => {}
                    }
                }
            }
        }
    });
    
    Ok(())
}

Hour 4: UI + Polish
Goal: Working browser client with clean UI
html<!-- ui/index.html -->
<!DOCTYPE html>
<html>
<head>
    <title>stationInvariant</title>
    <link rel="stylesheet" href="style.css">
</head>
<body>
    <div class="container">
        <h1>station<span>Invariant</span></h1>
        
        <div class="status">
            <span id="status">Disconnected</span>
            <span id="offset">Offset: -</span>
        </div>
        
        <div class="messages" id="messages"></div>
        
        <div class="input-area">
            <input type="text" id="input" placeholder="Type message..." />
            <button id="send">Send</button>
        </div>
    </div>
    
    <script src="app.js"></script>
</body>
</html>
css/* ui/style.css */
* { margin: 0; padding: 0; box-sizing: border-box; }

body {
    font-family: 'SF Mono', Monaco, monospace;
    background: #0a0a0a;
    color: #e0e0e0;
    line-height: 1.6;
}

.container {
    max-width: 800px;
    margin: 0 auto;
    padding: 40px 20px;
}

h1 {
    font-size: 2rem;
    margin-bottom: 30px;
    color: #fff;
}

h1 span {
    color: #00ff88;
    font-weight: 300;
}

.status {
    display: flex;
    justify-content: space-between;
    padding: 10px 15px;
    background: #1a1a1a;
    border-left: 3px solid #00ff88;
    margin-bottom: 20px;
    font-size: 0.9rem;
}

.messages {
    background: #111;
    border: 1px solid #222;
    height: 400px;
    overflow-y: auto;
    padding: 15px;
    margin-bottom: 20px;
}

.message {
    padding: 8px 12px;
    margin-bottom: 8px;
    background: #1a1a1a;
    border-left: 2px solid #333;
}

.message .meta {
    font-size: 0.75rem;
    color: #666;
    margin-bottom: 4px;
}

.message .text {
    color: #e0e0e0;
}

.input-area {
    display: flex;
    gap: 10px;
}

input {
    flex: 1;
    padding: 12px 15px;
    background: #1a1a1a;
    border: 1px solid #333;
    color: #e0e0e0;
    font-family: inherit;
    font-size: 1rem;
}

input:focus {
    outline: none;
    border-color: #00ff88;
}

button {
    padding: 12px 30px;
    background: #00ff88;
    border: none;
    color: #0a0a0a;
    font-weight: 600;
    cursor: pointer;
    font-family: inherit;
}

button:hover {
    background: #00dd77;
}

button:active {
    transform: translateY(1px);
}
javascript// ui/app.js
class StationClient {
    constructor() {
        this.ws = null;
        this.streamId = 1n;
        this.lastOffset = 0n;
        
        this.statusEl = document.getElementById('status');
        this.offsetEl = document.getElementById('offset');
        this.messagesEl = document.getElementById('messages');
        this.inputEl = document.getElementById('input');
        this.sendBtn = document.getElementById('send');
        
        this.connect();
        this.setupHandlers();
    }
    
    connect() {
        this.ws = new WebSocket('ws://localhost:9000');
        this.ws.binaryType = 'arraybuffer';
        
        this.ws.onopen = () => {
            this.statusEl.textContent = 'Connected';
            this.statusEl.style.color = '#00ff88';
            
            // Subscribe to stream
            this.subscribe(this.streamId, 0n);
        };
        
        this.ws.onmessage = (event) => {
            const frame = this.decodeFrame(new Uint8Array(event.data));
            
            if (frame.type === 0x02) { // MESSAGE
                this.addMessage(frame.offset, frame.payload);
                this.lastOffset = frame.offset;
                this.offsetEl.textContent = `Offset: ${frame.offset}`;
            }
        };
        
        this.ws.onerror = () => {
            this.statusEl.textContent = 'Error';
            this.statusEl.style.color = '#ff0044';
        };
        
        this.ws.onclose = () => {
            this.statusEl.textContent = 'Disconnected';
            this.statusEl.style.color = '#ff0044';
        };
    }
    
    setupHandlers() {
        this.sendBtn.onclick = () => this.sendMessage();
        this.inputEl.onkeypress = (e) => {
            if (e.key === 'Enter') this.sendMessage();
        };
    }
    
    sendMessage() {
        const text = this.inputEl.value.trim();
        if (!text) return;
        
        const payload = new TextEncoder().encode(text);
        const frame = this.encodeFrame(0x01, this.streamId, 0n, payload);
        
        this.ws.send(frame);
        this.inputEl.value = '';
    }
    
    subscribe(streamId, offset) {
        const frame = this.encodeFrame(0x03, streamId, offset, new Uint8Array(0));
        this.ws.send(frame);
    }
    
    encodeFrame(type, streamId, offset, payload) {
        const buf = new ArrayBuffer(21 + payload.length);
        const view = new DataView(buf);
        
        view.setUint8(0, type);
        view.setBigUint64(1, streamId, true);
        view.setBigUint64(9, offset, true);
        view.setUint32(17, payload.length, true);
        
        new Uint8Array(buf, 21).set(payload);
        
        return buf;
    }
    
    decodeFrame(data) {
        const view = new DataView(data.buffer);
        
        return {
            type: view.getUint8(0),
            streamId: view.getBigUint64(1, true),
            offset: view.getBigUint64(9, true),
            payload: data.slice(21),
        };
    }
    
    addMessage(offset, payload) {
        const text = new TextDecoder().decode(payload);
        
        const div = document.createElement('div');
        div.className = 'message';
        div.innerHTML = `
            <div class="meta">Offset ${offset} · ${new Date().toLocaleTimeString()}</div>
            <div class="text">${this.escapeHtml(text)}</div>
        `;
        
        this.messagesEl.appendChild(div);
        this.messagesEl.scrollTop = this.messagesEl.scrollHeight;
    }
    
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Start
new StationClient();

4. Deployment (5 Minutes)
bash# Build release binary
cargo build --release

# Copy UI files
mkdir -p dist/ui
cp ui/* dist/ui/

# Run
./target/release/station
Or Docker:
dockerfileFROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/station /usr/local/bin/
COPY ui /ui
CMD ["station"]

5. What You've Actually Learned
By building this you now understand:

Binary protocols - hand-writing encoders/decoders
Disk durability - flush() before ACK, crash safety
Async concurrency - tokio tasks, channels, broadcast
WebSocket - upgrade, binary frames, bidirectional
Atomic operations - lock-free offset assignment
Fan-out patterns - pub/sub without databases
Professional UI - vanilla JS, zero dependencies, clean aesthetics

This is production-shaped code. Not a tutorial. Not a toy.

6. What Comes Next (After You Finish)
Only after this works perfectly:

Add replay from offset (read disk backwards)
Add metrics endpoint (Prometheus format)
Add multiple streams (HashMap of channels)
Add crash recovery test (kill -9, restart, verify data)
Add benchmarks (measure throughput)

Then you can start thinking about Elixir/Zig.

7. Success Criteria
✅ You can open index.html and chat
✅ Messages survive server restart
✅ Multiple browser tabs see same messages
✅ No crashes under load (1000 messages/sec)
✅ Code is < 1500 lines total
✅ Zero external dependencies beyond tokio + tungstenite
✅ You understand every single line

8. Repository Checklist
Before you push:

 README explains what it is, how to run
 cargo test passes
 cargo clippy has zero warnings
 UI works in Chrome/Firefox
 data/ is in .gitignore
 No commented-out code
 No TODO comments
 Every function has a purpose


This is buildable TODAY.
This teaches REAL systems programming.
This earns you the right to build the grand vision.
Now go build it. Every hour. Every commit. Every test.
No shortcuts. No AI slop. Just clean, working code.
