use std::{path::PathBuf, vec};

use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use axum_extra::routing::SpaRouter;
use serde::Deserialize;
use sync_wrapper::SyncWrapper;

#[derive(Deserialize)]
struct SearchQuery {
    text: String,
}

async fn search_programs(Query(SearchQuery { text }): Query<SearchQuery>) -> impl IntoResponse {
    let programs = vec![
        "Marriott Bonvoy".to_owned(),
        "Best Western Rewards".to_owned(),
    ];

    if text.trim().is_empty() {
        return (StatusCode::OK, Json(vec![]));
    }

    let result = programs
        .into_iter()
        .filter(|p| p.to_lowercase().contains(&text.trim().to_lowercase()))
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(result))
}

#[shuttle_service::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder(folder = "public")] static_folder: PathBuf,
) -> shuttle_service::ShuttleAxum {
    let router = Router::new()
        .route("/api/programs/search", get(search_programs))
        .merge(SpaRouter::new("/", static_folder).index_file("index.html"));
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
