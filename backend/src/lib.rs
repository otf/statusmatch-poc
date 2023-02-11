use std::{path::PathBuf, vec};

use axum::{
    extract::{FromRef, Path, Query, State},
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

#[derive(Clone, FromRef)]
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

#[derive(Serialize)]
struct Link {
    program: String,
    status: String,
}

#[derive(Deserialize)]
struct SearchQuery {
    text: String,
}

async fn search_programs(
    State(pool): State<PgPool>,
    Query(SearchQuery { text }): Query<SearchQuery>,
) -> impl IntoResponse {
    if text.trim().is_empty() {
        return (StatusCode::OK, Json(vec![]));
    }

    let mut conn = pool.acquire().await.unwrap();

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

async fn get_statuses(State(pool): State<PgPool>, Path(id): Path<i32>) -> impl IntoResponse {
    let mut conn = pool.acquire().await.unwrap();

    let statuses = sqlx::query_as!(
        Status,
        "SELECT * FROM program_statuses WHERE program_id = $1",
        id,
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();
    (StatusCode::OK, Json(statuses))
}

async fn diagnose_links(
    State(pool): State<PgPool>,
    Path((id, level)): Path<(i32, i32)>,
) -> impl IntoResponse {
    let mut conn = pool.acquire().await.unwrap();

    let links = sqlx::query_as!(
        Link,
        r#"
            SELECT
                (SELECT
                    name
                    FROM programs
                    WHERE id = to_program_id
                ) AS "program!",
                (SELECT
                    name
                    FROM program_statuses
                    WHERE program_id = to_program_id
                    AND level = to_status_level
                ) AS "status!"
            FROM reports 
            WHERE 
                result = 'match'
                AND from_program_id = $1
                AND from_status_level = $2
        "#,
        id,
        level
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();
    (StatusCode::OK, Json(links))
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
        .route("/api/programs/:id/statuses", get(get_statuses))
        .route(
            "/api/programs/:id/statuses/:level/links",
            get(diagnose_links),
        )
        .merge(SpaRouter::new("/", static_folder).index_file("index.html"))
        .with_state(AppState { pool });
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
