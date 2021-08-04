use crate::index::add::add_index;
use crate::index::create::create_index;
use crate::index::search::search_index;
use serde::Serialize;
use serde_json::Value;

// use byteorder::{BigEndian, ReadBytesExt};
use serde::Deserialize;
use std::io::{Read, Result, Write};
use std::net::TcpStream;

#[derive(Serialize, Debug)]
pub enum Status {
    Ok,
    Wrong,
}

#[derive(Serialize, Debug)]
pub struct Message {
    pub(crate) status: Status,
    pub(crate) message: Option<Value>,
}

#[derive(Deserialize, PartialEq, Debug)]
enum Cmd {
    Create,
    Add,
    Search,
}

#[derive(Deserialize, PartialEq, Debug)]
struct RequestMessage {
    cmd: Cmd,
    body: String,
}

#[derive(Copy, Clone)]
pub struct TantivyServer;

impl Message {
    pub fn encode(self) -> Vec<u8> {
        let msg = serde_json::to_vec(&self).unwrap();
        let mut buf = Vec::with_capacity(4 + msg.len());
        if cfg!(target_endian = "big") {
            buf.extend_from_slice(&(msg.len() as u32).to_be_bytes());
        } else {
            buf.extend_from_slice(&(msg.len() as u32).to_le_bytes());
        }
        buf.extend_from_slice(&msg);
        buf
    }
}

impl TantivyServer {
    pub fn send(self, stream: &mut TcpStream, msg: Message) -> Result<()> {
        let data = &msg.encode();
        stream.write(&data).expect("write_all while receive error");
        stream.flush().expect("fail to flush");
        Ok(())
    }

    pub fn receive(self, stream: &mut TcpStream) -> Result<()> {
        let mut buf = [0u8; 4];
        stream.read_exact(&mut buf)?;
        let len: u32;
        if cfg!(target_endian = "big") {
            len = u32::from_be_bytes(buf);
        } else {
            len = u32::from_le_bytes(buf);
        }
        let mut buf: Vec<u8> = vec![0u8; len as usize];
        stream.read_exact(&mut buf)?;

        let msg = serde_json::from_slice::<RequestMessage>(&buf)?;
        match msg.cmd {
            Cmd::Create => {
                create_index(&msg.body)?;
            }
            Cmd::Add => {
                add_index(&msg.body)?;
            }
            Cmd::Search => {
                let res = search_index(&msg.body)?;
                self.send(
                    stream,
                    Message {
                        status: Status::Ok,
                        message: Some(serde_json::to_value(res).unwrap()),
                    },
                )?;
            }
        }
        Ok(())
    }
}
