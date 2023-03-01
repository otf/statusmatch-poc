use axum::Server;
use cachet::router;
use dotenv::dotenv;
use sqlx::PgPool;
use std::{env, net::SocketAddr, path::PathBuf};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let port = env::var("PORT").expect("PORT must be set").parse().unwrap();
    let service_url = env::var("SERVICE_URL").unwrap();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&db_url).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    let static_folder = PathBuf::from("public");

    let router = router(&service_url, pool, &static_folder);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap()
}
