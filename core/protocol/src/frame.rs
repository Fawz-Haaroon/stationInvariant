pub struct Record {
    pub offset: u64,
    pub payload: Vec<u8>,
}

impl Record {
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + 4 + self.payload.len());
        buf.extend_from_slice(&self.offset.to_le_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);
        buf
    }
}
