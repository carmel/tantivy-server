use serde::{Deserialize, Serialize};
pub(crate) mod add;
pub(crate) mod create;
mod jieba_tokenizer;
pub(crate) mod search;

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct AddConf {
    is_merge: bool,
    thread_num: usize,
    buffer_size: usize, // size in kb
}
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct IndexConf {
    base_dir: String,
    add: AddConf,
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
