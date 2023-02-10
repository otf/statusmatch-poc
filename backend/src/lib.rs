use std::{path::PathBuf, vec};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use serde::{Deserialize, Serialize};
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use sync_wrapper::SyncWrapper;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

#[derive(Serialize)]
struct Program {
    id: i32,
    name: String,
}

#[derive(Serialize)]
struct Status {
    program_id: i32,
    level: i32,
    name: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    text: String,
}

async fn search_programs(
    state: State<AppState>,
    Query(SearchQuery { text }): Query<SearchQuery>,
) -> impl IntoResponse {
    if text.trim().is_empty() {
        return (StatusCode::OK, Json(vec![]));
    }

    let mut conn = state.pool.acquire().await.unwrap();

    let programs = sqlx::query_as!(
        Program,
        "SELECT * FROM programs WHERE LOWER(name) LIKE LOWER($1)",
        format!("%{}%", text.trim()),
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();

    (StatusCode::OK, Json(programs))
}

#[derive(Deserialize)]
struct FindByProgramId {
    program_id: i32,
}

async fn find_by_program_id(
    state: State<AppState>,
    Query(FindByProgramId { program_id }): Query<FindByProgramId>,
) -> impl IntoResponse {
    let mut conn = state.pool.acquire().await.unwrap();

    let statuses = sqlx::query_as!(
        Status,
        "SELECT * FROM program_statuses WHERE program_id = $1",
        program_id,
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();
    (StatusCode::OK, Json(statuses))
}

#[shuttle_service::main]
async fn axum(
    #[shuttle_shared_db::Postgres(local_uri = "{secrets.DATABASE_URL}")] pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
    #[shuttle_static_folder::StaticFolder(folder = "public")] static_folder: PathBuf,
) -> shuttle_service::ShuttleAxum {
    std::env::set_var("DATABASE_URL", secret_store.get("DATABASE_URL").unwrap());
    sqlx::migrate!().run(&pool).await.unwrap();

    let router = Router::new()
        .route("/api/programs/search", get(search_programs))
        .route("/api/statuses/find", get(find_by_program_id))
        .merge(SpaRouter::new("/", static_folder).index_file("index.html"))
        .with_state(AppState { pool });
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
