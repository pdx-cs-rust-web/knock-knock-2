mod error;
mod joke;
mod templates;

use error::*;
use joke::*;
use templates::*;

extern crate mime;

use axum::{self, extract::State, response, routing};
use clap::Parser;
extern crate fastrand;
use sqlx::SqlitePool;
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
}

struct AppState {
    db: SqlitePool,
}

async fn get_joke(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let app_state = app_state.read().await;
    let db = &app_state.db;
    let joke = sqlx::query_as!(Joke, "SELECT * FROM jokes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
        .unwrap();
    let joke = IndexTemplate::joke(&joke);
    response::Html(joke.to_string())
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db = SqlitePool::connect("sqlite://db/knock-knock.db").await?;
    sqlx::migrate!().run(&db).await?;
    if let Some(path) = args.init_from {
        let jokes = read_jokes(path)?;
        let mut tx = db.begin().await?;
        for j in &jokes {
            sqlx::query!(
                "INSERT INTO jokes VALUES ($1, $2);",
                j.whos_there,
                j.answer_who,
            )
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
    }
    let state = Arc::new(RwLock::new(AppState { db }));

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kk2=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // https://carlosmv.hashnode.dev/adding-logging-and-tracing-to-an-axum-app-rust
    let trace_layer = trace::TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::new().level(tracing::Level::INFO))
        .on_response(trace::DefaultOnResponse::new().level(tracing::Level::INFO));

    let mime_favicon = "image/vnd.microsoft.icon".parse().unwrap();
    let app = axum::Router::new()
        .route("/", routing::get(get_joke))
        .route_service(
            "/knock.css",
            services::ServeFile::new_with_mime("assets/static/knock.css", &mime::TEXT_CSS_UTF_8),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime("assets/static/favicon.ico", &mime_favicon),
        )
        .layer(trace_layer)
        .with_state(state);

    let listener = net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        eprintln!("kk2: error: {}", err);
        std::process::exit(1);
    }
}
