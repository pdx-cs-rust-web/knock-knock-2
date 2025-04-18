mod error;
mod joke;
mod templates;

use error::*;
use joke::*;
use templates::*;

extern crate mime;

use axum::{self, extract::State, response, routing};
extern crate fastrand;
use tokio::{net, sync::RwLock};
use tower_http::{services, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::Arc;

struct AppState {
    jokes: Vec<Joke>,
}

async fn get_joke(State(app_state): State<Arc<RwLock<AppState>>>) -> response::Html<String> {
    let app_state = app_state.read().await;
    let njokes = app_state.jokes.len();
    let i = fastrand::usize(0..njokes);
    let joke = &app_state.jokes[i];
    let joke = IndexTemplate::joke(joke);
    response::Html(joke.to_string())
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let jokes = read_jokes("assets/static/jokes.json")?;
    let state = Arc::new(RwLock::new(AppState{jokes}));

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
            services::ServeFile::new_with_mime(
                "assets/static/knock.css",
                &mime::TEXT_CSS_UTF_8,
            ),
        )
        .route_service(
            "/favicon.ico",
            services::ServeFile::new_with_mime(
                "assets/static/favicon.ico",
                &mime_favicon,
            ),
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
