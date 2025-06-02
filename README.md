# knock-knock-2: A Pure Rust Knock-Knock Joke Webserver, iteration 2
Bart Massey 2025-04

This… thing uses a Tokio/Axum/Askama/Sqlx/Sqlite stack to
serve knock-knock jokes.

# Build and Run

By default the joke database URI is
`sqlite://db/knock-knock.db`. You can override this with the
`DATABASE_URL` environment variable or with the `--db-uri`
command-line argument.

To build and run this code for the first time, you will
probably want:

    cargo run --release -- --init-from assets/static/jokes.json

This will load an initial collection of jokes into the
newly-created database.

## Development

For working on the code, you will want to

    cargo install sqlx-cli

* `sqlx` migrations are turned on, with reverse
  sequential migrations. Add a migration called `<name>` with

        sqlx migrate add -r -s <name>

  and then edit the migration files. You may want to run
  
        sqlx migrate run

* `sqlx` compile-time checking of queries against
  the database schemas is turned on. If you modify the
  database schemas or the queries in the source code, please
  run

        DATABASE_URL=sqlite://db/knock-knock.db cargo sqlx prepare

  to update things.

Because of the above you need to

    git add .sqlx migrations

before committing to ensure things are up to date.

## Docker

Install Docker CE and build an image with

      docker build -t kk2 .

You can then run the image with

      docker run -p 3000:3000 kk2

Note that the image is built with the database from the
build directory copied in: it will not persist across image
builds. A named Docker volume could be used for persisting
the database.

## License

This work is made available under the "Apache 2.0 or MIT
License". See the file `LICENSE.txt` in this distribution for
license terms.
