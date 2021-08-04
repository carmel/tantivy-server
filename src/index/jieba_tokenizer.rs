//! A library that bridges between tantivy and jieba-rs.
//!
//! It implements a [`JiebaTokenizer`](./struct.JiebaTokenizer.html) for the purpose.
#![forbid(unsafe_code)]

use crate::{JIEBA, STOP_WORD};
use jieba_rs;

use tantivy::tokenizer::{BoxTokenStream, Token, TokenStream, Tokenizer};

/// Tokenize the text using jieba_rs.
///
/// Need to load dict on first tokenization.
///
/// # Example
/// ```rust
/// use tantivy::tokenizer::*;
/// let tokenizer = tantivy_jieba::JiebaTokenizer {};
/// let mut token_stream = tokenizer.token_stream("测试");
/// assert_eq!(token_stream.next().unwrap().text, "测试");
/// assert!(token_stream.next().is_none());
/// ```
///
/// # Register tantivy tokenizer
/// ```rust
/// use tantivy::schema::Schema;
/// use tantivy::tokenizer::*;
/// use tantivy::Index;
/// # fn main() {
/// # let schema = Schema::builder().build();
/// let tokenizer = tantivy_jieba::JiebaTokenizer {};
/// let index = Index::create_in_ram(schema);
/// index.tokenizers()
///      .register("jieba", tokenizer);
/// # }
#[derive(Clone)]
pub struct JiebaTokenizer;

/// Token stream instantiated by [`JiebaTokenizer`](./struct.JiebaTokenizer.html).
///
/// Use [`JiebaTokenizer::token_stream`](./struct.JiebaTokenizer.html#impl-Tokenizer<%27a>).
pub struct JiebaTokenStream {
    tokens: Vec<Token>,
    index: usize,
}

impl TokenStream for JiebaTokenStream {
    fn advance(&mut self) -> bool {
        if self.index < self.tokens.len() {
            self.index = self.index + 1;
            true
        } else {
            false
        }
    }

    fn token(&self) -> &Token {
        &self.tokens[self.index - 1]
    }

    fn token_mut(&mut self) -> &mut Token {
        &mut self.tokens[self.index - 1]
    }
}

impl Tokenizer for JiebaTokenizer {
    fn token_stream<'a>(&self, text: &'a str) -> BoxTokenStream<'a> {
        let mut indices = text.char_indices().collect::<Vec<_>>();
        indices.push((text.len(), '\0'));
        let orig_tokens = JIEBA.tokenize(text, jieba_rs::TokenizeMode::Search, true);
        let mut tokens = Vec::new();

        for i in 0..orig_tokens.len() {
            let token = &orig_tokens[i];
            if STOP_WORD.contains(&token.word.to_string()) {
                continue;
            }
            tokens.push(Token {
                offset_from: token.start,
                offset_to: token.end,
                position: token.start,
                text: String::from(&text[(indices[token.start].0)..(indices[token.end].0)]),
                position_length: token.end - token.start,
            });
        }

        // if let Some(words) = STOP_WORD. {}
        // println!("?#:", STOP_WORD);
        // match STOP_WORD {
        //     Some(word) => {
        //         for i in 0..orig_tokens.len() {
        //             let token = &orig_tokens[i];
        //             if word.contains(token.word) {
        //                 continue;
        //             }
        //             tokens.push(Token {
        //                 offset_from: token.start,
        //                 offset_to: token.end,
        //                 position: token.start,
        //                 text: String::from(&text[(indices[token.start].0)..(indices[token.end].0)]),
        //                 position_length: token.end - token.start,
        //             });
        //         }
        //     }
        //     None => {
        //         for i in 0..orig_tokens.len() {
        //             let token = &orig_tokens[i];
        //             tokens.push(Token {
        //                 offset_from: token.start,
        //                 offset_to: token.end,
        //                 position: token.start,
        //                 text: String::from(&text[(indices[token.start].0)..(indices[token.end].0)]),
        //                 position_length: token.end - token.start,
        //             });
        //         }
        //     }
        // };

        BoxTokenStream::from(JiebaTokenStream { tokens, index: 0 })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use tantivy::tokenizer::*;
        let tokenizer = crate::index::jieba_tokenizer::JiebaTokenizer {};
        let mut token_stream = tokenizer.token_stream(
            "张华考上了北京大学；李萍进了中等技术学校；我在百货公司当售货员：我们都有光明的前途",
        );
        let mut tokens = Vec::new();
        while let Some(token) = token_stream.next() {
            tokens.push(token.text.clone());
        }
        println!("{:?}", tokens);
        // assert_eq!(
        //     tokens,
        //     vec![
        //         "张华",
        //         "考上",
        //         "了",
        //         "北京",
        //         "大学",
        //         "北京大学",
        //         "；",
        //         "李萍",
        //         "进",
        //         "了",
        //         "中等",
        //         "技术",
        //         "术学",
        //         "学校",
        //         "技术学校",
        //         "；",
        //         "我",
        //         "在",
        //         "百货",
        //         "公司",
        //         "百货公司",
        //         "当",
        //         "售货",
        //         "货员",
        //         "售货员",
        //         "：",
        //         "我们",
        //         "都",
        //         "有",
        //         "光明",
        //         "的",
        //         "前途"
        //     ]
        // );
        token_stream = tokenizer.token_stream("主耶稣曾说，祂所说的话就是灵，就是生命（约六63）。我们能想象到，这本是神的话的圣经就是灵么？牠不仅是白纸上的黑字；牠是一些更高、更深、更丰富且更丰满的东西──那就是灵和生命。圣经又告诉我们，那灵就是神自己（约四24），而生命也就是基督（约十四6）。");
        let mut tokens = Vec::new();
        while let Some(token) = token_stream.next() {
            tokens.push(token.text.clone());
        }
        println!("{:?}", tokens);
    }
}
