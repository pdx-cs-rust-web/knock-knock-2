mod error;
mod joke;
mod templates;

use error::*;
use joke::*;
use templates::*;

extern crate log;
extern crate mime;

use axum::{
    self,
    extract::{Query, State},
    http,
    response::{self, IntoResponse},
    routing,
};
use clap::Parser;
extern crate fastrand;
use serde::Deserialize;
use sqlx::{Row, SqlitePool, migrate::MigrateDatabase, sqlite};
use tokio::{net, sync::RwLock};
use tokio_stream::StreamExt;
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::borrow::Cow;
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

#[derive(Deserialize)]
struct GetJokeParams {
    id: Option<String>,
    tags: Option<String>,
}

async fn tagged_joke(db: &SqlitePool, tags: &str) -> Result<Option<String>, sqlx::Error> {
    let mut jtx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qtags;").execute(&mut *jtx).await?;
    sqlx::query("CREATE TEMPORARY TABLE qtags (tag VARCHR(200));")
        .execute(&mut *jtx)
        .await?;
    for tag in tags.split(',') {
        sqlx::query("INSERT INTO qtags VALUES ($1);")
            .bind(tag)
            .execute(&mut *jtx)
            .await?;
    }
    let joke_ids = sqlx::query("SELECT DISTINCT joke_id FROM tags JOIN qtags ON tags.tag = qtags.tag ORDER BY RANDOM() LIMIT 1;")
        .fetch_all(&mut *jtx)
        .await?;
    let njoke_ids = joke_ids.len();
    let result = if njoke_ids == 1 {
        Some(joke_ids[0].get(0))
    } else {
        None
    };
    jtx.commit().await?;

    Ok(result)
}

async fn get_joke(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetJokeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_state = app_state.write().await;
    let db = app_state.db.clone();

    // Specified.
    if let GetJokeParams { id: Some(id), .. } = params {
        let joke_result = sqlx::query_as!(Joke, "SELECT * FROM jokes WHERE id = $1;", id)
            .fetch_one(&db)
            .await;
        let result = match joke_result {
            Ok(joke) => {
                let mut tags =
                    sqlx::query_scalar!("SELECT tag FROM tags WHERE joke_id = $1;", joke.id)
                        .fetch(&db);
                let mut tag_list: Vec<String> = Vec::new();
                while let Some(tag) = tags.next().await {
                    let tag = tag.unwrap_or_else(|e| {
                        log::error!("tag fetch failed: {}", e);
                        panic!("tag fetch failed")
                    });
                    tag_list.push(tag);
                }
                let tag_string = tag_list.join(", ");

                app_state.current_joke = joke.clone();
                let joke = IndexTemplate::new(joke.clone(), tag_string);
                Ok(response::Html(joke.to_string()).into_response())
            }
            Err(e) => {
                log::warn!("joke fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        };
        return result;
    }

    if let GetJokeParams { tags: Some(tags), .. } = params {
        log::info!("joke tags: {}", tags);

        let mut tags_string = String::new();
        for c in tags.chars() {
            if c.is_alphabetic() || c == ',' {
                let cl: String = c.to_lowercase().collect();
                tags_string.push_str(&cl);
            }
        }

        let joke_result = tagged_joke(&db, &tags_string).await;
        match joke_result {
            Ok(Some(id)) => {
                let uri = format!("/?id={}", id);
                return Ok(response::Redirect::to(&uri).into_response());
            }
            Ok(None) => {
                log::info!("tagged joke selection was empty");
            }
            Err(e) => {
                log::error!("tagged joke selection database error: {}", e);
                panic!("tagged joke selection database error");
            }
        }
    }

    // Random.
    let joke_result = sqlx::query_scalar!("SELECT id FROM jokes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(&db)
        .await;
    match joke_result {
        Ok(id) => {
            let uri = format!("/?id={}", id);
            Ok(response::Redirect::to(&uri).into_response())
        }
        Err(e) => {
            log::error!("joke selection failed: {}", e);
            panic!("joke selection failed");
        }
    }
}

fn get_db_uri(db_uri: Option<&str>) -> Cow<str> {
    if let Some(db_uri) = db_uri {
        db_uri.into()
    } else if let Ok(db_uri) = std::env::var("DATABASE_URL") {
        db_uri.into()
    } else {
        "sqlite://db/knock-knock.db".into()
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
        'next_joke: for jj in jokes {
            let mut jtx = db.begin().await?;
            let (j, ts) = jj.to_joke();
            let joke_insert = sqlx::query!(
                "INSERT INTO jokes (id, whos_there, answer_who, joke_source) VALUES ($1, $2, $3, $4);",
                j.id,
                j.whos_there,
                j.answer_who,
                j.joke_source,
            )
            .execute(&mut *jtx)
            .await;
            if let Err(e) = joke_insert {
                eprintln!("error: joke insert: {}: {}", j.id, e);
                jtx.rollback().await?;
                continue;
            };
            for t in ts {
                let tag_insert =
                    sqlx::query!("INSERT INTO tags (joke_id, tag) VALUES ($1, $2);", j.id, t,)
                        .execute(&mut *jtx)
                        .await;
                if let Err(e) = tag_insert {
                    eprintln!("error: tag insert: {} {}: {}", j.id, t, e);
                    jtx.rollback().await?;
                    continue 'next_joke;
                };
            }
            jtx.commit().await?;
        }
        return Ok(());
    }
    let current_joke = Joke {
        id: "mojo".to_string(),
        whos_there: "Mojo".to_string(),
        answer_who: "Mo' jokes, please.".to_string(),
        joke_source: "Unknown".to_string(),
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
