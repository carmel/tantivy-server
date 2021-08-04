use index::IndexConf;
use lazy_static::lazy_static;
use log::{error, info};
use log4rs;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Write};
use std::net::{Shutdown, TcpListener};
use std::thread;

pub mod index;
mod server;
mod util;

use crate::server::{Message, Status, TantivyServer};

mod error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    bind_addr: String,
    log_config: String,
    index: IndexConf,
}

lazy_static! {

    static ref RE: Regex = Regex::new(r"([[:word:]]+):").unwrap();

    static ref CONF: Config = {
        let f = File::open("config/app.yml").unwrap();
        serde_yaml::from_reader(f).unwrap()
    };

    static ref HANDSHAKE: HashMap<String, String> = {
      let mut handshake: HashMap<String,String> = HashMap::with_capacity(2);
      handshake.insert("greeting".to_string(), "Tantivy Search Engine 1.0".to_string());

      if cfg!(target_endian = "big") {
        handshake.insert("target_endian".to_string(), "Big".to_string());
      } else {
        handshake.insert("target_endian".to_string(), "Little".to_string());
      }
      handshake
    };

    static ref JIEBA: jieba_rs::Jieba = {
      // println!("{}", std::env::current_dir().unwrap().display());
      match File::open(CONF.index.tokenizer.jieba.dict_path.to_string()) {
          Ok(r) => jieba_rs::Jieba::with_dict(&mut BufReader::new(r)).unwrap(),
          Err(_) => jieba_rs::Jieba::new(),
      }
    };

    static ref STOP_WORD: Vec<String> = {
      let mut words: Vec<String> = Vec::new();
      if let Ok(r) = File::open(CONF.index.tokenizer.jieba.stop_word_path.to_string()) {
        let lines = BufReader::new(r).lines();
        for line in lines {
          if (&line).is_ok()  {
            let word = line.as_ref().unwrap().trim();
            if !word.is_empty() {
              words.push(word.to_string());
            }
          }
        }
      }
      words
    };
}

fn main() {
    log4rs::init_file(CONF.log_config.to_string(), Default::default()).unwrap();
    // let config = Config::parse();
    let listener = TcpListener::bind(&CONF.bind_addr)
        .expect(&format!("faild to listen: {}", &CONF.bind_addr).to_string());
    info!("Server started: {}", CONF.bind_addr);

    let server = TantivyServer {};

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    stream
                        .write(&serde_json::to_vec(&*HANDSHAKE).unwrap())
                        .expect("fail to handshake");
                    loop {
                        println!("looping...");
                        match &server.receive(&mut stream) {
                            Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                                error!("timeout, err={:?}", e);
                                stream.shutdown(Shutdown::Both).unwrap();
                                break;
                            }
                            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                                error!("client disconnected, err={:?}", e);
                                stream.shutdown(Shutdown::Both).unwrap();
                                break;
                            }
                            Err(e) => {
                                error!("receive err={:?}", e);
                                &server.send(
                                    &mut stream,
                                    Message {
                                        status: Status::Wrong,
                                        message: Some(serde_json::to_value(e.to_string()).unwrap()),
                                    },
                                );
                                stream.shutdown(Shutdown::Both).unwrap();
                                break;
                            }
                            Ok(_) => {
                                &server.send(
                                    &mut stream,
                                    Message {
                                        status: Status::Ok,
                                        message: None,
                                    },
                                );
                                break;
                            }
                        }
                    }
                });
            }
            Err(e) => {
                error!("incoming: {}", e);
            }
        }
    }
}
