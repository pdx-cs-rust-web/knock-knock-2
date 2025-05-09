use crate::*;

use std::collections::HashSet;
use std::ops::Deref;
use std::path::Path;

use crate::KnockKnockError;

use serde::Deserialize;

#[derive(Debug, Serialize, Deserialize)]
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
    pub fn new(joke: Joke, tags: Vec<String>) -> Self {
        let tags = tags.into_iter().collect();
        Self {
            id: joke.id,
            whos_there: joke.whos_there,
            answer_who: joke.answer_who,
            tags,
            source: joke.joke_source,
        }
    }

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

impl axum::response::IntoResponse for &JsonJoke {
    fn into_response(self) -> axum::response::Response {
        (http::StatusCode::OK, axum::Json(&self)).into_response()
    }
}

pub async fn get(db: &SqlitePool, joke_id: &str) -> Result<(Joke, Vec<String>), sqlx::Error> {
    let joke = sqlx::query_as!(Joke, "SELECT * FROM jokes WHERE id = $1;", joke_id)
        .fetch_one(db)
        .await?;

    let tags: Vec<String> = sqlx::query_scalar!("SELECT tag FROM tags WHERE joke_id = $1;", joke_id)
        .fetch_all(db)
        .await?;

    Ok((joke, tags))
}

pub async fn get_tagged<'a, I>(db: &SqlitePool, tags: I) -> Result<Option<String>, sqlx::Error>
    where I: Iterator<Item=&'a str>
{
    let mut jtx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qtags;").execute(&mut *jtx).await?;
    sqlx::query("CREATE TEMPORARY TABLE qtags (tag VARCHR(200));")
        .execute(&mut *jtx)
        .await?;
    for tag in tags {
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

pub async fn get_random(db: &SqlitePool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar!("SELECT id FROM jokes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(db)
        .await
}
