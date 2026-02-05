use std::io::Read;
use std::net::TcpStream;

use crate::Server;
use core_protocol::frame::{Frame, FrameType};

pub fn handle(mut stream: TcpStream, server: &mut Server) -> std::io::Result<()> {
    let mut buf = Vec::new();

    loop {
        let mut tmp = [0u8; 4096];
        let n = stream.read(&mut tmp)?;

        if n == 0 {
            break;
        }

        buf.extend_from_slice(&tmp[..n]);


        loop {
            match Frame::try_decode(&mut buf) {
                Ok(Some(frame)) => {
                    // normal path: handle the frame
                    server.handle_frame(frame, &mut stream)?;
                }

                Ok(None) => {
                    // need more bytes
                    break;
                }

                Err(err) => {
                    // protocol violation — client sent garbage
                    // decision: close connection
                    eprintln!("protocol error: {:?}", err);
                    return Ok(());
                }
            }
        }
    }



    Ok(())
}

