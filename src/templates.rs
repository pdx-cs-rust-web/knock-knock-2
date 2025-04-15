use crate::*;

use askama::Template;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    joke: &'a Joke,
    stylesheet: &'static str,
}

impl<'a> IndexTemplate<'a> {
    pub fn joke(joke: &'a Joke) -> Self {
        Self {
            joke,
            stylesheet: "/knock.css",
        }
    }
}
