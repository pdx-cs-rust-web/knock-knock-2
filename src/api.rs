use crate::*;

async fn get_joke_by_id(db: &SqlitePool, joke_id: &str) -> Result<response::Response, http::StatusCode> {
    let joke_result = joke::get(db, joke_id).await;
    match joke_result {
        Ok((joke, tags)) => Ok(JsonJoke::new(joke, tags).into_response()),
        Err(e) => {
            log::warn!("joke fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_joke(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(joke_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    get_joke_by_id(db, &joke_id).await
}

pub async fn get_tagged_joke(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Json(tags): Json<Vec<String>>,
) -> Result<response::Response, http::StatusCode> {
    log::info!("get tagged joke: {:?}", tags);
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let joke_result = joke::get_tagged(db, tags.iter().map(String::as_ref)).await;
    match joke_result {
        Ok(Some(joke_id)) => get_joke_by_id(db, &joke_id).await,
        Ok(None) => {
            log::warn!("joke tag fetch failed tagging");
            Err(http::StatusCode::NOT_FOUND)
        }
        Err(e) => {
            log::warn!("joke tag fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}

pub async fn get_random_joke(
    State(app_state): State<Arc<RwLock<AppState>>>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    let joke_result = joke::get_random(db).await;
    match joke_result {
        Ok(joke_id) => get_joke_by_id(db, &joke_id).await,
        Err(e) => {
            log::warn!("get random joke failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}
