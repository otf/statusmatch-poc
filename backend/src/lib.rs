use std::{path::PathBuf, vec};

use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use axum_extra::routing::SpaRouter;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shuttle_secrets::SecretStore;
use sqlx::PgPool;
use sync_wrapper::SyncWrapper;

#[derive(Serialize)]
struct Status {
    id: i32,
    name: String,
}

impl Status {
    fn new(id: i32, name: &str) -> Self {
        Self {
            id,
            name: name.to_owned(),
        }
    }
}

#[derive(Serialize)]
struct Programs {
    id: i32,
    name: String,
    statuses: Vec<Status>,
}

impl Programs {
    fn new(id: i32, name: &str, statuses: Vec<Status>) -> Self {
        Self {
            id,
            name: name.to_owned(),
            statuses,
        }
    }
}

fn create_programs() -> Vec<Programs> {
    vec![
        Programs::new(
            0,
            "Marriott Bonvoy",
            vec![
                Status::new(0, "Member"),
                Status::new(1, "Silver Elite"),
                Status::new(2, "Gold Elite"),
            ],
        ),
        Programs::new(
            1,
            "Best Western Rewards",
            vec![
                Status::new(3, "Blue"),
                Status::new(4, "Gold"),
                Status::new(5, "Plutinum"),
            ],
        ),
    ]
}

#[derive(Deserialize)]
struct SearchQuery {
    text: String,
}

async fn search_programs(Query(SearchQuery { text }): Query<SearchQuery>) -> impl IntoResponse {
    let programs = create_programs();

    if text.trim().is_empty() {
        return (StatusCode::OK, Json(vec![]));
    }

    let program_jsons = programs
        .into_iter()
        .filter(|p| p.name.to_lowercase().contains(&text.trim().to_lowercase()))
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name
            })
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(program_jsons))
}

#[derive(Deserialize)]
struct FindByProgramId {
    program_id: i32,
}

async fn find_by_program_id(
    Query(FindByProgramId { program_id }): Query<FindByProgramId>,
) -> impl IntoResponse {
    let programs = create_programs();

    let statuses = programs
        .into_iter()
        .find(|p| p.id == program_id)
        .unwrap_or_else(|| panic!("The program is not found. {}", program_id)) // Todo: should returns 404
        .statuses;

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
        .merge(SpaRouter::new("/", static_folder).index_file("index.html"));
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
