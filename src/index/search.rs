use crate::{CONF, RE};
use serde::Deserialize;
use serde_json::Value;
use std::io::{Error, ErrorKind, Result};
use std::{
    collections::{HashMap, HashSet},
    path::Path,
};
use tantivy::{
    collector::{Count, TopDocs},
    query::QueryParser,
    schema::Field,
    schema::FieldType,
    Document, Index, SnippetGenerator,
};

use super::jieba_tokenizer;

#[derive(Deserialize, Debug)]
pub struct IndexQuery {
    index: String,
    param: String,
    size: usize,
    offset: usize,
}

pub fn search_index(query_json: &str) -> Result<HashMap<String, Value>> {
    let mut index_query = serde_json::from_str::<IndexQuery>(query_json)?;

    if index_query.size > 120 {
        index_query.size = 120;
    }
    let index = Index::open_in_dir(Path::new(&CONF.index.base_dir).join(index_query.index))
        .map_err(|e| Error::new(ErrorKind::Other, format!("Index open_in_dir: {}", e)))?;
    index
        .tokenizers()
        .register("jieba", jieba_tokenizer::JiebaTokenizer {});
    let schema = index.schema();
    let default_fields: Vec<Field> = schema
        .fields()
        .filter(|&(_, field_entry)| match field_entry.field_type() {
            FieldType::Str(ref text_field_options) => {
                if let Some(opt) = text_field_options.get_indexing_options() {
                    opt.tokenizer() != "raw"
                } else {
                    false
                }
                // text_field_options.get_indexing_options().is_some()
            }
            _ => false,
        })
        .map(|(field, _)| field)
        .collect();
    let query_parser = QueryParser::new(
        schema.clone(),
        default_fields.clone(),
        index.tokenizers().clone(),
    );
    // let query_parser = QueryParser::for_index(&index, vec![title, body]);
    let reader = index
        .reader()
        .map_err(|e| Error::new(ErrorKind::Other, format!("Index reader: {}", e)))?;

    let query = query_parser
        .parse_query(&index_query.param)
        .expect("Parsing the query failed");
    let searcher = reader.searcher();
    let (top_docs, count) = {
        searcher
            .search(
                &query,
                &(
                    TopDocs::with_limit(index_query.size).and_offset(index_query.offset),
                    Count,
                ),
            )
            .map_err(|e| Error::new(ErrorKind::Other, format!("Searcher search: {}", e)))?
    };

    let mut snippet_map: HashMap<String, SnippetGenerator> = HashMap::new();
    {
        let query_field = extract_field(&index_query.param);

        for f in &default_fields {
            let fname = schema.get_field_name(*f).to_string();
            if query_field.contains(&fname) {
                snippet_map.insert(
                    fname,
                    SnippetGenerator::create(&searcher, &*query, *f).map_err(|e| {
                        Error::new(ErrorKind::Other, format!("SnippetGenerator create: {}", e))
                    })?,
                );
            }
        }
    }

    let mut result: HashMap<String, Value> = HashMap::with_capacity(2);
    result.insert("Total".to_string(), serde_json::to_value(count).unwrap());
    result.insert(
        "Data".to_string(),
        serde_json::to_value::<Vec<HashMap<String, Value>>>(
            top_docs
                .iter()
                .map(|(_, doc_address)| {
                    // .map(|(score, doc_address)| {
                    let doc: Document = searcher.doc(*doc_address).unwrap();
                    let mut result: HashMap<String, Value> = HashMap::new();

                    let named_doc = schema.to_named_doc(&doc).0;
                    let mut raw_field = HashMap::new();
                    for f in named_doc.keys() {
                        if !default_fields.contains(&schema.get_field(&f.to_string()).unwrap()) {
                            // raw_field.remove(&f.to_string());
                            raw_field.insert(f.to_string(), named_doc[f].get(0));
                        }
                    }

                    result.insert(
                        "RawField".to_string(),
                        serde_json::to_value(raw_field).unwrap(),
                    );

                    let mut snippet: HashMap<String, Value> = HashMap::new();
                    for (f, g) in snippet_map.iter() {
                        snippet.insert(
                            f.to_string(),
                            serde_json::to_value(g.snippet_from_doc(&doc).to_html()).unwrap(),
                        );
                    }

                    result.insert(
                        "Snippet".to_string(),
                        serde_json::to_value(snippet).unwrap(),
                    );

                    result
                })
                .collect(),
        )
        .unwrap(),
    );

    Ok(result)
}

fn extract_field(input: &str) -> HashSet<String> {
    let mut field: HashSet<String> = HashSet::new();
    for (_, c) in RE.captures_iter(input).enumerate() {
        field.insert(c[1].to_string());
    }
    field
}

#[test]
fn test_search_index() {
    let query = "{\"index\":\"book\",\"param\":\"Text:\\\"圣经\\\"\",\"size\":20,\"offset\":0}";

    match search_index(query) {
        Ok(res) => {
            println!("{:#?}", res);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
