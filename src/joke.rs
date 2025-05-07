use crate::*;

use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::KnockKnockError;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct JsonJoke {
    id: String,
    whos_there: String,
    answer_who: String,
    tags: HashSet<String>,
    source: String,
}

#[derive(Clone)]
pub struct Joke {
    pub id: String,
    pub whos_there: String,
    pub answer_who: String,
    pub joke_source: String,
}

pub fn read_jokes<P: AsRef<Path>>(jokes_path: P) -> Result<Vec<JsonJoke>, KnockKnockError> {
    let f = std::fs::File::open(jokes_path.as_ref())?;
    let jokes = serde_json::from_reader(f)?;
    Ok(jokes)
}

impl JsonJoke {
    pub fn to_joke(&self) -> (Joke, impl Iterator<Item = &str>) {
        let joke = Joke {
            id: self.id.clone(),
            whos_there: self.whos_there.clone(),
            answer_who: self.answer_who.clone(),
            joke_source: self.source.clone(),
        };
        let tags = self.tags.iter().map(String::deref);
        (joke, tags)
    }
}

pub async fn get(db: &SqlitePool, joke_id: &str) -> Result<(Joke, Vec<String>), sqlx::Error> {
    let joke = sqlx::query_as!(Joke, "SELECT * FROM jokes WHERE id = $1;", joke_id)
        .fetch_one(db)
        .await?;

    type Tags = Vec<String>;
    let tags: Tags = sqlx::query_scalar!("SELECT tag FROM tags WHERE joke_id = $1;", joke_id)
        .fetch_all(db)
        .await?;

    Ok((joke, tags))
}
