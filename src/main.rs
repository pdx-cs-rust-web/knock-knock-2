mod error;
mod joke;
mod templates;

use error::*;
use joke::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{self, extract::State, response, routing};
use clap::Parser;
extern crate fastrand;
use sqlx::{SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

#[derive(Parser)]
struct Args {
    #[arg(short, long, name = "init-from")]
    init_from: Option<std::path::PathBuf>,
    #[arg(short, long, name = "db-uri")]
    db_uri: Option<String>,
}

struct AppState {
    db: SqlitePool,
    current_joke: Joke,
}

async fn get_joke(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let mut app_state = app_state.write().await;
    let db = &app_state.db;
    let joke_result = sqlx::query_as!(Joke, "SELECT * FROM jokes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await;
    match joke_result {
        Ok(joke) => app_state.current_joke = joke,
        Err(e) => log::warn!("joke fetch failed: {}", e),
    }
    let joke = IndexTemplate::joke(&app_state.current_joke);
    response::Html(joke.to_string())
}

fn get_db_uri(db_uri: Option<&str>) -> String {
    if let Some(db_uri) = db_uri {
        db_uri.to_string()
    } else if let Ok(db_uri) = std::env::var("KK2_DB_URI") {
        db_uri
    } else {
        "sqlite://db/knock-knock.db".to_string()
    }
}

fn extract_db_dir(db_uri: &str) -> Result<&str, KnockKnockError> {
    if db_uri.starts_with("sqlite://") && db_uri.ends_with(".db") {
        let start = db_uri.find(':').unwrap() + 3;
        let mut path = &db_uri[start..];
        if let Some(end) = path.rfind('/') {
            path = &path[..end];
        } else {
            path = "";
        }
        Ok(path)
    } else {
        Err(KnockKnockError::InvalidDbUri(db_uri.to_string()))
    }
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let db_uri = get_db_uri(args.db_uri.as_deref());
    if !sqlite::Sqlite::database_exists(&db_uri).await? {
        let db_dir = extract_db_dir(&db_uri)?;
        std::fs::create_dir_all(db_dir)?;
        sqlite::Sqlite::create_database(&db_uri).await?
    }

    let db = SqlitePool::connect(&db_uri).await?;
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
        return Ok(());
    }
    let current_joke = Joke {
        whos_there: "Mojo".to_string(),
        answer_who: "Mo' jokes, please.".to_string(),
    };
    let app_state = AppState { db, current_joke };
    let state = Arc::new(RwLock::new(app_state));

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
