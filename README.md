# tantivy-server

This is a [tanitvy](https://github.com/tantivy-search/tantivy) based search engine tcp server, project reference [tantivy-cli](https://github.com/tantivy-search/tantivy-cli) implementation.

It also extends the following features:

1. some parameters can be freely configured in the app.yml file.
2. use `jieba-rs` tokenizer for Chinese.
3. Chinese word tokenizer can be customized with user dict and stop words.

> The project is my first rust project, if you find that the code is not well written, please feel free to correct me.

Finally, thanks to @PSeitz for helping me to solve the error about Chinese word tokenizer. [tantivy/issue#1134](https://github.com/tantivy-search/tantivy/issues/1134), [tantivy-jieba/issues#4](https://github.com/jiegec/tantivy-jieba/issues/4).
