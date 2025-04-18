use std::path::Path;

use crate::KnockKnockError;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Joke {
    pub whos_there: String,
    pub answer_who: String,
}

pub fn read_jokes<P: AsRef<Path>>(jokes_path: P) -> Result<Vec<Joke>, KnockKnockError> {
    let f = std::fs::File::open(jokes_path.as_ref())?;
    let jokes = serde_json::from_reader(f)?;
    Ok(jokes)
}
