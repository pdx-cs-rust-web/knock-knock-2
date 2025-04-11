use axum::{self, response, routing};
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
        r#"<p style="font-weight: bold">Knock Knock!</p>
<p>Who's there?</p>
<p style="font-weight: bold">{}</p>
<p>{} who?</p>
<p style="font-weight: bold">{}</p>"#,
        joke.whos_there,
        joke.whos_there,
        joke.answer,
    )
}

async fn hello() -> response::Html<String> {
    let joke = render_joke(&THE_JOKE);
    response::Html(format!(r#"<head><title>"Knock Knock!"</title></head><body>{}</body></html>"#, joke))
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let app = axum::Router::new().route("/",  routing::get(hello));
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
