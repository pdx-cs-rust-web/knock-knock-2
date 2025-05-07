use crate::*;

#[derive(Deserialize)]
pub struct GetJokeParams {
    id: Option<String>,
    tags: Option<String>,
}

async fn tagged_joke(db: &SqlitePool, tags: &str) -> Result<Option<String>, sqlx::Error> {
    let mut jtx = db.begin().await?;
    sqlx::query("DROP TABLE IF EXISTS qtags;").execute(&mut *jtx).await?;
    sqlx::query("CREATE TEMPORARY TABLE qtags (tag VARCHR(200));")
        .execute(&mut *jtx)
        .await?;
    for tag in tags.split(',') {
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

pub async fn get_joke(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Query(params): Query<GetJokeParams>,
) -> Result<response::Response, http::StatusCode> {
    let mut app_writer = app_state.write().await;
    let db = app_writer.db.clone();

    // Specified.
    if let GetJokeParams { id: Some(id), .. } = params {
        let joke_result = joke::get(&db, &id).await;
        let result = match joke_result {
            Ok((joke, tags)) => {
                let tag_string = tags.join(", ");

                app_writer.current_joke = joke.clone();
                let joke = IndexTemplate::new(joke.clone(), tag_string);
                Ok(response::Html(joke.to_string()).into_response())
            }
            Err(e) => {
                log::warn!("joke fetch failed: {}", e);
                Err(http::StatusCode::NOT_FOUND)
            }
        };
        return result;
    }

    if let GetJokeParams { tags: Some(tags), .. } = params {
        log::info!("joke tags: {}", tags);

        let mut tags_string = String::new();
        for c in tags.chars() {
            if c.is_alphabetic() || c == ',' {
                let cl: String = c.to_lowercase().collect();
                tags_string.push_str(&cl);
            }
        }

        let joke_result = tagged_joke(&db, &tags_string).await;
        match joke_result {
            Ok(Some(id)) => {
                let uri = format!("/?id={}", id);
                return Ok(response::Redirect::to(&uri).into_response());
            }
            Ok(None) => {
                log::info!("tagged joke selection was empty");
            }
            Err(e) => {
                log::error!("tagged joke selection database error: {}", e);
                panic!("tagged joke selection database error");
            }
        }
    }

    // Random.
    let joke_result = sqlx::query_scalar!("SELECT id FROM jokes ORDER BY RANDOM() LIMIT 1;")
        .fetch_one(&db)
        .await;
    match joke_result {
        Ok(id) => {
            let uri = format!("/?id={}", id);
            Ok(response::Redirect::to(&uri).into_response())
        }
        Err(e) => {
            log::error!("joke selection failed: {}", e);
            panic!("joke selection failed");
        }
    }
}
