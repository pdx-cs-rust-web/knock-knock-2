use crate::*;

pub async fn get_joke(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(joke_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_writer = app_state.write().await;
    let db = &app_writer.db;
    let joke_result = joke::get(db, &joke_id).await;
    match joke_result {
        Ok((joke, tags)) => Ok(JsonJoke::new(joke, tags).into_response()),
        Err(e) => {
            log::warn!("joke fetch failed: {}", e);
            Err(http::StatusCode::NOT_FOUND)
        }
    }
}
