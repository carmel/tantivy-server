use crate::CONF;
use std::{
    io::{Error, ErrorKind, Result},
    path::Path,
};

use serde::{Deserialize, Serialize};
use tantivy::{Index, IndexWriter};
pub(crate) mod add;
pub(crate) mod create;
pub(crate) mod delete;
mod jieba_tokenizer;
pub(crate) mod search;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct IndexConf {
    base_dir: String,
    is_merge: bool,
    thread_num: usize,
    total_heap_size: usize, // size in mb
    pub tokenizer: TokenizerConf,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct TokenizerConf {
    pub jieba: Tokenizer,
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct Tokenizer {
    pub dict_path: String,
    pub stop_word_path: String,
}

fn get_index(index: String) -> Result<Index> {
    Index::open_in_dir(Path::new(&CONF.index.base_dir).join(index))
        .map_err(|e| Error::new(ErrorKind::Other, format!("Index open_in_dir: {}", e)))
}

fn get_index_writer(index: &Index) -> Result<IndexWriter> {
    index
        // .writer_with_num_threads(
        //     CONF.index.thread_num,
        //     CONF.index.total_heap_size * 1024 * 1024,
        // )
        .writer(CONF.index.total_heap_size * 1024 * 1024)
        .map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Index writer_with_num_threads: {}", e),
            )
        })
}
