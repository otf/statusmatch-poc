use std::path::PathBuf;

use axum::{routing::get, Router};
use axum_extra::routing::SpaRouter;
use sync_wrapper::SyncWrapper;

async fn hello_world() -> &'static str {
    "Hello, world! from Rust Backend"
}

#[shuttle_service::main]
async fn axum(
    #[shuttle_static_folder::StaticFolder(folder = "public")] static_folder: PathBuf,
) -> shuttle_service::ShuttleAxum {
    let router = Router::new()
        .route("/api/hello", get(hello_world))
        .merge(SpaRouter::new("/", static_folder).index_file("index.html"));
    let sync_wrapper = SyncWrapper::new(router);

    Ok(sync_wrapper)
}
