use crate::CONF;
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use tantivy::{merge_policy::NoMergePolicy, Index};

use super::jieba_tokenizer;

#[derive(Deserialize, Serialize, Debug)]
pub struct IndexData {
    index: String,
    data: Vec<Map<String, Value>>,
}

pub fn add_index(index_json: &str) -> Result<()> {
    let json_index = serde_json::from_str::<IndexData>(index_json)?;

    let index = Index::open_in_dir(Path::new(&CONF.index.base_dir).join(json_index.index))
        .map_err(|e| Error::new(ErrorKind::Other, format!("Index open_in_dir: {}", e)))?;

    index
        .tokenizers()
        .register("jieba", jieba_tokenizer::JiebaTokenizer {});

    let schema = index.schema();

    let schema_clone = schema.clone();

    for m in json_index.data {
        let data = serde_json::to_string(&m)?;
        match schema_clone.parse_document(&data) {
            Ok(doc) => {
                let mut thread_num = 1;
                if CONF.index.add.thread_num != 0 {
                    thread_num = CONF.index.add.thread_num;
                }

                let buffer_size_per_thread = CONF.index.add.buffer_size / thread_num;

                // let mut index_writer = index.writer(buffer_size_per_thread);
                let mut index_writer = index
                    .writer_with_num_threads(thread_num, buffer_size_per_thread)
                    .map_err(|e| {
                        Error::new(
                            ErrorKind::Other,
                            format!("Index writer_with_num_threads: {}", e),
                        )
                    })?;

                if CONF.index.add.is_merge {
                    index_writer.set_merge_policy(Box::new(NoMergePolicy));
                }
                index_writer.add_document(doc);

                let index_result = index_writer.commit();

                match index_result {
                    Ok(docstamp) => {
                        info!("Commit succeed, docstamp at {}", docstamp);
                        // info!("Waiting for merging threads");
                        index_writer.wait_merging_threads().map_err(|e| {
                            Error::new(ErrorKind::Other, format!("wait_merging_threads: {}", e))
                        })?;
                    }
                    Err(e) => {
                        index_writer.rollback().unwrap();
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("index_writer rollback: {}", e),
                        ));
                    }
                }
            }
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("DocParsingError: {}", e),
                ));
            }
        }
    }
    Ok(())
}

#[test]
fn test_add_index() {
    use std::fs;
    // use std::fs::File;
    // use std::io::BufRead;
    // use std::io::BufReader;

    // let read_file = File::open("test_index/wikipedia.json").unwrap();
    // let reader = BufReader::new(Box::new(read_file));

    // for article_line in reader.lines() {
    //     println!("{}", article_line.unwrap());
    // }

    let s = fs::read_to_string(std::path::PathBuf::from("test_index/wikipedia.json")).unwrap();

    let data_json = IndexData {
        index: "test_index/wikipedia".to_string(),
        data: serde_json::from_str::<Vec<Map<String, Value>>>(&s).unwrap(),
    };

    println!(
        "{:?}",
        add_index(&serde_json::to_string(&data_json).unwrap())
    );

    // let path = PathBuf::from("test_index/wikipedia");
    // let index = Index::open_in_dir(&path).unwrap();
    // let segments = index.searchable_segment_ids().unwrap();
    // const HEAP_SIZE: usize = 300_000_000;
    // index.writer(HEAP_SIZE).unwrap().merge(&segments);

    // Index::open_in_dir(&path)
    //     .unwrap()
    //     .writer_with_num_threads(1, 40_000_000)
    //     .unwrap()
    //     .garbage_collect_files();
}
