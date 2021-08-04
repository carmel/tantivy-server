use crate::CONF;
use std::io::{ErrorKind, Result};
use std::path::Path;
use std::{fs, io::Error};

use serde::Deserialize;
use tantivy::{schema::*, Index};

#[derive(Deserialize, Debug)]
pub struct IndexSchema {
    index: String,
    field: Vec<FieldSchema>,
}

#[derive(Deserialize, PartialEq, Debug)]
enum Record {
    Basic,
    Freq,
    Position,
}
#[derive(Deserialize, PartialEq, Debug)]
enum Tokenizer {
    EnStem,
    Jieba,
}
#[derive(Deserialize, Debug)]
struct FieldOption {
    stored: bool,
    fast: bool,
    indexed: bool,
    record: Option<Record>, // basic/freq/position
}
#[derive(Deserialize, Debug)]
pub struct FieldSchema {
    name: String,
    typ: String,
    tokenizer: Option<Tokenizer>,
    option: FieldOption,
}

impl FieldSchema {
    fn ask_add_field_text(self, schema_builder: &mut SchemaBuilder) {
        let mut text_options = TextOptions::default();
        if self.option.stored {
            text_options = text_options.set_stored();
        }

        if self.option.indexed {
            let mut text_indexing_options = TextFieldIndexing::default()
                .set_index_option(IndexRecordOption::Basic)
                .set_tokenizer(match self.tokenizer {
                    Some(t) => match t {
                        Tokenizer::EnStem => "en_stem",
                        Tokenizer::Jieba => "jieba",
                    },
                    None => "raw",
                });
            // .set_tokenizer("en_stem");
            match self.option.record {
                Some(r) => match r {
                    Record::Basic => (),
                    Record::Freq => {
                        text_indexing_options =
                            text_indexing_options.set_index_option(IndexRecordOption::WithFreqs)
                    }
                    Record::Position => {
                        text_indexing_options = text_indexing_options
                            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
                    }
                },
                None => (),
            }

            text_options = text_options.set_indexing_options(text_indexing_options);
        }

        schema_builder.add_text_field(&self.name, text_options);
    }

    fn ask_add_num_field_with_options(self, schema_builder: &mut SchemaBuilder) {
        let mut int_options = IntOptions::default();
        if self.option.stored {
            int_options = int_options.set_stored();
        }
        if self.option.fast {
            int_options = int_options.set_fast(Cardinality::SingleValue);
        }
        if self.option.indexed {
            int_options = int_options.set_indexed();
        }
        match self.typ.to_ascii_uppercase().as_str() {
            "U64" => {
                schema_builder.add_u64_field(&self.name, int_options);
            }
            "F64" => {
                schema_builder.add_f64_field(&self.name, int_options);
            }
            "I64" => {
                schema_builder.add_i64_field(&self.name, int_options);
            }
            "Date" => {
                schema_builder.add_date_field(&self.name, int_options);
            }
            _ => {
                // We only pass to this function if the field type is numeric
                unreachable!();
            }
        }
    }

    fn ask_add_field_bytes(self, schema_builder: &mut SchemaBuilder) {
        let mut bytes_options = BytesOptions::default();
        if self.option.stored {
            bytes_options = bytes_options.set_stored();
        }

        if self.option.indexed {
            bytes_options = bytes_options.set_indexed();
        }

        schema_builder.add_bytes_field(&self.name, bytes_options);
    }
}

pub fn create_index(schema_json: &str) -> Result<()> {
    let json_schema = serde_json::from_str::<IndexSchema>(schema_json)?;
    // println!("{:#?}", json_schema);
    let mut schema_builder = SchemaBuilder::default();
    for f in json_schema.field {
        if is_valid_field_name(&f.name) {
            match f.typ.to_ascii_uppercase().as_str() {
                "TEXT" => {
                    f.ask_add_field_text(&mut schema_builder);
                }
                "U64" | "I64" | "F64" | "DATE" => {
                    f.ask_add_num_field_with_options(&mut schema_builder);
                }
                "FACET" => {
                    schema_builder.add_facet_field(&f.name, tantivy::schema::INDEXED);
                }
                "BYTES" => {
                    f.ask_add_field_bytes(&mut schema_builder);
                }
                _ => {
                    f.ask_add_field_text(&mut schema_builder);
                }
            }
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                "Field name must match the pattern [_a-zA-Z0-9]+",
            ));
        }
    }

    let directory = &Path::new(&CONF.index.base_dir).join(json_schema.index);
    match fs::create_dir_all(directory) {
        Ok(_) => (),
        // Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => (),
        Err(e) => {
            return Err(e);
        }
    }
    let schema = schema_builder.build();
    Index::create_in_dir(&directory, schema)
        .map_err(|e| Error::new(ErrorKind::Other, format!("Index create_in_dir: {}", e)));

    // index.tokenizers().register(
    //     "jieba",
    //     TextAnalyzer::from(jieba_tokenizer::JiebaTokenizer {}).filter(StopWordFilter::remove(
    //         BufReader::new(r)
    //             .lines()
    //             .filter_map(io::Result::ok)
    //             .collect(),
    //     )),
    // );

    // index
    //     .tokenizers()
    //     .register("jieba", jieba_tokenizer::JiebaTokenizer {});
    // StopWordFilter::remove(stop_word.iter().map(|&s| s.to_string()).collect())
    Ok(())
}

#[test]
fn test_create_index() {
    let data = r#"
    {
      "index": "wikipedia",
      "field":
        [{
              "name": "title",
              "typ": "TEXT",
              "tokenizer": "EnStem",
              "option": {
                "stored": true,
                "fast": false,
                "indexed": true,
                "record": "Position"
              }
          },{
              "name": "body",
              "typ": "TEXT",
              "tokenizer": "EnStem",
              "option": {
                "stored": true,
                "fast": false,
                "indexed": true,
                "record": "Position"
              }
          },{
              "name": "url",
              "typ": "TEXT",
              "option": {
                "stored": true,
                "fast": true,
                "indexed": true
              }
          }]
        }"#;

    println!("{:?}", create_index(data));
}
