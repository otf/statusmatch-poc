use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    scraper::renew_statuses(&db_url).await;
}
