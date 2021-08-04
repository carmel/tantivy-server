# tantivy-server

This is a [tanitvy](https://github.com/tantivy-search/tantivy) based search engine tcp server. project reference [tantivy-cli](https://github.com/tantivy-search/tantivy-cli) implementation. The project is my first rust project, if you find that the code is not well written, please feel free to correct me.

It also extends the following features:

1. some parameters can be freely configured in the app.yml file.
2. use `jieba-rs` tokenizer for Chinese.
3. Chinese word tokenizer can be customized with user words and stop words.
