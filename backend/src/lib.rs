use async_stream::stream;
use futures::stream::StreamExt;
use std::{path::PathBuf, vec};
use tower_http::services::ServeDir;

use axum::{
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use bech32::ToBase32;
use secp256k1::{ecdsa::Signature, Message, PublicKey, Secp256k1};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
mod auth;
use auth::{AuthError, Claims};

type Challenge = String;
type ServiceUrl = String;

#[derive(Clone, FromRef)]
struct AppState {
    pool: PgPool,
    service_url: ServiceUrl,
}

#[derive(Serialize, sqlx::Type)]
struct Program {
    id: i32,
    name: String,
}

#[derive(Serialize, sqlx::Type)]
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

#[derive(Deserialize)]
struct Credential {
    program_id: i32,
    username: String,
    password: String,
}

#[derive(Serialize)]
struct UserStatus {
    program: Program,
    status: Status,
}

async fn get_user_statuses(
    Claims { sub, .. }: Claims,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let mut conn = pool.acquire().await.unwrap();

    let pubkey = hex::decode(&sub).unwrap();

    let credentials = sqlx::query_as!(
        Credential,
        r#"
        SELECT
            program_id,
            username,
            password
        FROM
            user_credentials 
        WHERE
            user_pubkey = $1
            AND program_id = 147
        "#,
        &pubkey,
    )
    .fetch_all(&mut conn)
    .await
    .unwrap();

    let statuses = stream! {
        for credential in credentials {
            let status =
                dormys::retrieve_status(&credential.username, &credential.password).unwrap();
            let user_status = sqlx::query_as!(UserStatus, r#"
                SELECT
                    (
                        programs.id,
                        programs.name
                    ) AS "program!: Program",
                    (
                        program_statuses.program_id,
                        program_statuses.level,
                        program_statuses.name
                    ) AS "status!: Status"
                FROM program_statuses
                INNER JOIN programs ON program_statuses.program_id = programs.id
                WHERE
                    programs.id = $1 AND
                    program_statuses.name = $2
                "#,
                credential.program_id,
                &status,
            )
            .fetch_optional(&mut conn)
            .await
            .unwrap();

            yield user_status
        }
    };

    let resp = json!(statuses.collect::<Vec<_>>().await);

    (StatusCode::OK, Json(resp))
}

async fn login(
    State(service_url): State<ServiceUrl>,
    State(pool): State<PgPool>,
) -> impl IntoResponse {
    let challenge: [u8; 32] = rand::random();
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query!("INSERT INTO challenges (challenge) VALUES($1)", &challenge)
        .fetch_optional(&mut conn)
        .await
        .unwrap();

    let k1 = hex::encode(challenge);
    let url = format!("{}/api/auth?tag=login&k1={}", &service_url, &k1);
    let encoded = bech32::encode("lnurl", url.to_base32(), bech32::Variant::Bech32).unwrap();

    let resp = json!({
        "lnurl": encoded,
        "k1": k1,
    });

    (StatusCode::OK, Json(resp))
}

async fn get_login_status(
    State(pool): State<PgPool>,
    Path(k1): Path<Challenge>,
) -> impl IntoResponse {
    let k1 = hex::decode(&k1).unwrap();
    let mut conn = pool.acquire().await.unwrap();
    let pubkey = sqlx::query_scalar!(
        r#"
        SELECT user_pubkey AS "pubkey!" FROM challenges WHERE challenge = $1
        "#,
        &k1,
    )
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    if let Some(pubkey) = pubkey {
        let auth = auth::authorize(pubkey).unwrap();
        auth.into_response()
    } else {
        AuthError::WaitingForLogin.into_response()
    }
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

        sqlx::query!(
            "INSERT INTO users (pubkey) VALUES ($1) ON CONFLICT DO NOTHING",
            &key
        )
        .fetch_optional(&mut trans)
        .await
        .unwrap();

        sqlx::query!(
            "UPDATE challenges SET user_pubkey = $1 WHERE challenge = $2",
            &key,
            &k1,
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

pub fn router(service_url: &str, pool: PgPool, static_folder: &PathBuf) -> Router {
    Router::new()
        .route("/api/login", get(login))
        .route("/api/login/:k1", get(get_login_status))
        .route("/api/auth", get(auth))
        .route("/api/user/statuses", get(get_user_statuses))
        .route("/api/programs/search", get(search_programs))
        .route("/api/programs/:id/statuses", get(get_statuses))
        .route(
            "/api/programs/:id/statuses/:level/links",
            get(diagnose_links),
        )
        .merge(Router::new().nest_service("/", ServeDir::new(static_folder)))
        .with_state(AppState {
            service_url: service_url.to_string(),
            pool,
        })
}
