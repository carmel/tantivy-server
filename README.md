# tantivy-server

This is a [tanitvy](https://github.com/quickwit-inc/tantivy) based search engine tcp server, project reference [tantivy-cli](https://github.com/quickwit-inc/tantivy-cli) implementation.You can use a tcp connection to create, add, search and delete indexes.

It also extends the following features:

1. some parameters can be freely configured in the app.yml file.
2. use `jieba-rs` tokenizer for Chinese.
3. Chinese word tokenizer can be customized with user words and stop words.

## build server

```sh
cargo build --release --target=x86_64-unknown-linux-musl
```
