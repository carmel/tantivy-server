use std::io::{Error, ErrorKind, Result};

use serde::{Deserialize, Serialize};
use tantivy::Term;

use super::{get_index, get_index_writer};

#[derive(Deserialize, Serialize, Debug)]
struct QueryItem {
    index: String,
    field: String,
    text: String,
}

// todo: get_index函数需要包装SchemaBuilder

pub fn delete_index(query: &str) -> Result<()> {
    let item = serde_json::from_str::<QueryItem>(query)?;
    if item.index == "" {
        return Err(Error::new(
            ErrorKind::Other,
            "delete_index: index could not be empty!",
        ));
    }
    let index = get_index(item.index)?;

    let mut index_writer = get_index_writer(&index)?;
    if item.field != "" {
        if item.text == "" {
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "delete_index: field {} text could not be empty!",
                    item.field
                ),
            ));
        }

        if let Some(f) = index.schema().get_field(&item.field) {
            index_writer.delete_term(Term::from_field_text(f, &item.text));
        } else {
            return Err(Error::new(
                ErrorKind::Other,
                format!("delete_index: field {} not exist!", item.field),
            ));
        }
    } else {
        index_writer.delete_all_documents().unwrap();
    }
    index_writer.commit().unwrap();
    Ok(())
}

#[test]
fn test_delete_index() {
    // let query = "{\"index\":\"wikipedia\",\"param\":\"title:\\\"Vado\\\" AND (url:\\\"https://en.wikipedia.org/wiki?curid=48693283\\\" OR body:\\\"Vado\\\")\",\"size\":20,\"offset\":0}";
    let query = "{\"index\":\"book\",\"field\":\"BookId\",\"text\":\"l1\"}";

    match delete_index(query) {
        Ok(res) => {
            println!("{:#?}", res);
        }
        Err(e) => {
            println!("{}", e);
        }
    }
}
