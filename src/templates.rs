use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    joke: Joke,
    stylesheet: &'static str,
}

impl IndexTemplate {
    pub fn joke(joke: Joke) -> Self {
        Self {
            joke,
            stylesheet: "/knock.css",
        }
    }
}
