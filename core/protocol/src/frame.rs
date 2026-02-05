use std::convert::TryInto;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameType {
    Publish   = 0x01,
    Message   = 0x02,
    Subscribe = 0x03,
    Ack       = 0x04,
}

impl FrameType {
    fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(FrameType::Publish),
            0x02 => Some(FrameType::Message),
            0x03 => Some(FrameType::Subscribe),
            0x04 => Some(FrameType::Ack),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub frame_type: FrameType,
    pub stream_id:  u64,
    pub offset:     u64,
    pub payload:    Vec<u8>,
}

impl Frame {
    pub const HEADER_LEN: usize = 1 + 8 + 8 + 4;

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::HEADER_LEN + self.payload.len());

        buf.push(self.frame_type as u8);
        buf.extend_from_slice(&self.stream_id.to_le_bytes());
        buf.extend_from_slice(&self.offset.to_le_bytes());
        buf.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&self.payload);

        buf
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() < Self::HEADER_LEN {
            return Err(DecodeError::TooShort);
        }

        let frame_type =
            FrameType::from_u8(bytes[0]).ok_or(DecodeError::UnknownFrameType)?;

        let stream_id = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
        let offset    = u64::from_le_bytes(bytes[9..17].try_into().unwrap());
        let len       = u32::from_le_bytes(bytes[17..21].try_into().unwrap()) as usize;

        if bytes.len() != Self::HEADER_LEN + len {
            return Err(DecodeError::LengthMismatch);
        }

        let payload = bytes[21..].to_vec();

        Ok(Self {
            frame_type,
            stream_id,
            offset,
            payload,
        })
    }

    pub fn try_decode(buf: &mut Vec<u8>) -> Result<Option<Self>, DecodeError> {
        if buf.len() < Self::HEADER_LEN {
            return Ok(None);
        }

        let len =
            u32::from_le_bytes(buf[17..21].try_into().unwrap()) as usize;
        let total = Self::HEADER_LEN + len;

        if buf.len() < total {
            return Ok(None);
        }

        let frame = Self::decode(&buf[..total])?;
        buf.drain(..total);

        Ok(Some(frame))
    }
}

#[derive(Debug)]
pub enum DecodeError {
    TooShort,
    UnknownFrameType,
    LengthMismatch,
}

