use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    joke: Joke,
    stylesheet: &'static str,
    tags: String,
}

impl IndexTemplate {
    pub fn new(joke: Joke, tags: String) -> Self {
        Self {
            joke,
            stylesheet: "/knock.css",
            tags,
        }
    }
}
