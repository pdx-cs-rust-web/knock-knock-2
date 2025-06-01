use crate::*;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "kk2", description = "Knock-Knock Joke API")
    )
)]
pub struct ApiDoc;

pub fn router() -> OpenApiRouter<Arc<RwLock<AppState>>> {
    OpenApiRouter::new()
        .routes(routes!(get_joke))
        .routes(routes!(get_tagged_joke))
        .routes(routes!(get_random_joke))
        .routes(routes!(register))
        .routes(routes!(add_joke))
}

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

#[utoipa::path(
    get,
    path = "/joke/{joke_id}",
    responses(
        (status = 200, description = "Get a joke by id", body = [JsonJoke]),
        (status = 404, description = "No matching joke"),
    )
)]
pub async fn get_joke(
    State(app_state): State<Arc<RwLock<AppState>>>,
    Path(joke_id): Path<String>,
) -> Result<response::Response, http::StatusCode> {
    let app_reader = app_state.read().await;
    let db = &app_reader.db;
    get_joke_by_id(db, &joke_id).await
}

#[utoipa::path(
    get,
    path = "/tagged-joke",
    responses(
        (status = 200, description = "Get a joke by tags", body = [JsonJoke]),
        (status = 404, description = "No matching jokes"),
    )
)]
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

#[utoipa::path(
    get,
    path = "/random-joke",
    responses(
        (status = 200, description = "Get a random joke", body = [JsonJoke]),
        (status = 404, description = "No joke"),
    )
)]
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

#[utoipa::path(
    post,
    path = "/register",
    request_body(
        content = inline(authjwt::Registration),
        description = "Get an API key",
    ),
    responses(
        (status = 200, description = "JSON Web Token", body = authjwt::AuthBody),
        (status = 401, description = "Registration failed", body = authjwt::AuthError),
    )
)]
pub async fn register(
    State(appstate): State<SharedAppState>,
    Json(registration): Json<authjwt::Registration>,
) -> axum::response::Response {
    let appstate = appstate.read().await;
    match authjwt::make_jwt_token(&appstate, &registration) {
        Err(e) => e.into_response(),
        Ok(token) => (StatusCode::OK, token).into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/add-joke",
    request_body(
        content = inline(JsonJoke),
        description = "Joke to add"
    ),
    responses(
        (status = 201, description = "Added joke", body = ()),
        (status = 400, description = "Bad request", body = String),
        (status = 401, description = "Auth Error", body = authjwt::AuthError),
    )
)]
pub async fn add_joke(
    _claims: authjwt::Claims,
    State(appstate): State<SharedAppState>,
    Json(joke): Json<JsonJoke>,
) -> axum::response::Response {
    let appstate = appstate.read().await;
    match joke::add(&appstate.db, joke).await {
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
        Ok(()) => StatusCode::CREATED.into_response(),
    }
}
