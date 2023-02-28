use std::{path::PathBuf, vec};

use axum::{
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use axum_extra::routing::SpaRouter;
use bech32::ToBase32;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

async fn login(State(pool): State<PgPool>) -> impl IntoResponse {
    let challenge: [u8; 32] = rand::random();
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query!("INSERT INTO challenges VALUES($1)", &challenge)
        .fetch_optional(&mut conn)
        .await
        .unwrap();

    let k1 = hex::encode(challenge);
    let url = format!("https://cachet.shuttleapp.rs/api/auth?tag=login&k1={}", &k1);
    let encoded = bech32::encode("lnurl", url.to_base32(), bech32::Variant::Bech32).unwrap();

    let resp = json!({
        "lnurl": encoded,
    });

    (StatusCode::OK, Json(resp))
}

#[derive(Deserialize)]
struct LnurlAuth {
    k1: String,
    sig: String,
    key: String,
}

async fn auth(
    State(pool): State<PgPool>,
    Query(LnurlAuth { k1, sig, key }): Query<LnurlAuth>,
) -> impl IntoResponse {
    let k1 = hex::decode(&k1).unwrap();
    let sig = hex::decode(&sig).unwrap();
    let key = hex::decode(&key).unwrap();

    {
        let mut trans = pool.begin().await.unwrap();

        let count = sqlx::query_scalar!(
            "SELECT COUNT(challenge) FROM challenges WHERE challenge = $1",
            &k1
        )
        .fetch_one(&mut trans)
        .await
        .unwrap();

        if let Some(0) = count {
            let resp = json!({
                "status": "ERROR",
                "reason": "Challenge is not found.",
            });
            return (StatusCode::OK, Json(resp));
        }

        sqlx::query!("DELETE FROM challenges WHERE challenge = $1", &k1)
            .fetch_optional(&mut trans)
            .await
            .unwrap();

        sqlx::query!(
            "INSERT INTO users (pubkey) VALUES ($1) ON CONFLICT DO NOTHING",
            &key
        )
        .fetch_optional(&mut trans)
        .await
        .unwrap();

        trans.commit().await.unwrap();
    }

    let secp = Secp256k1::verification_only();
    let msg = Message::from_slice(&k1).unwrap();
    let sig = Signature::from_der(&sig).unwrap();
    let pk = PublicKey::from_slice(&key).unwrap();
    secp.verify_ecdsa(&msg, &sig, &pk).unwrap();

    let resp = json!({
        "status": "OK",
    });
    (StatusCode::OK, Json(resp))
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
                    AND level = MAX(to_status_level)
                ) AS "status!"
            FROM reports 
            WHERE 
                result = 'match'
                AND from_program_id = $1
                AND from_status_level <= $2
            GROUP BY to_program_id
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
        .route("/api/login", get(login))
        .route("/api/auth", get(auth))
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
