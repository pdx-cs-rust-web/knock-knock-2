# knock-knock-2: A Pure Rust Knock-Knock Joke Webserver, iteration 2
Bart Massey 2025-04

This… thing uses a Tokio/Axum/Askama/Sqlx/Sqlite stack to
serve knock-knock jokes.

# Build and Run

* `cargo install sqlx-cli`
* `mkdir db && sqlx database create --database-url sqlite://db/knock-knock.db`
* `cargo run -- --init-from assets/static/jokes.json`

This last line is optional, but will initialize the database
with a bunch of jokes.

## License

This work is made available under the "Apache 2.0 or MIT
License". See the file `LICENSE.txt` in this distribution for
license terms.
