use axum::{self, response, routing};
use axum_extra::response as extra_response;
use tokio::net;

struct Joke {
    whos_there: &'static str,
    answer: &'static str,
}

const THE_JOKE: Joke = Joke {
    whos_there: "Boo",
    answer: "You don't have to cry about it!",
};

fn render_joke(joke: &Joke) -> String {
    format!(
        r#"<p class="knock">Knock Knock!</p>
<p>Who's there?</p>
<p class="knock">{}</p>
<p>{} who?</p>
<p class="knock">{}</p>"#,
        joke.whos_there,
        joke.whos_there,
        joke.answer,
    )
}

async fn get_joke() -> response::Html<String> {
    let joke = render_joke(&THE_JOKE);
    response::Html(format!(r#"<head><title>"Knock Knock!"</title><link rel="stylesheet" href="/knock.css"></head><body>{}</body></html>"#, joke))
}

async fn get_css() -> extra_response::Css<&'static str> {
    extra_response::Css(r#".knock {font-weight: bold;}"#)
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let app = axum::Router::new()
        .route("/",  routing::get(get_joke))
        .route("/knock.css", routing::get(get_css));
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
