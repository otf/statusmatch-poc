use serde::Deserialize;
use sqlx::PgPool;

mod cocoweb;
mod dormys;

#[derive(Deserialize)]
struct Credential {
    user_pubkey: Vec<u8>,
    program_id: i32,
    username: String,
    password: String,
}

pub async fn renew_statuses(db_url: &str) {
    let pool = PgPool::connect(&db_url).await.unwrap();

    let credentials = sqlx::query_as!(Credential, "SELECT * FROM user_credentials",)
        .fetch_all(&pool)
        .await
        .unwrap();

    for credential in credentials {
        let status = if credential.program_id == 147 {
            dormys::retrieve_status(&credential.username, &credential.password).unwrap()
        } else if credential.program_id == 148 {
            cocoweb::retrieve_status(&credential.username, &credential.password).unwrap()
        } else {
            unimplemented!("Unimpemented Program")
        };

        sqlx::query!(
            r#"
            WITH get_level AS (
                SELECT level
                FROM program_statuses
                WHERE
                    program_id = $2
                    AND name = $3
            )
            INSERT INTO user_statuses
            VALUES (
                $1,
                $2,
                (SELECT * FROM get_level)
            )
            ON CONFLICT (user_pubkey, program_id)
            DO UPDATE
                SET level = (SELECT * FROM get_level)
            "#,
            &credential.user_pubkey,
            credential.program_id,
            status,
        )
        .fetch_optional(&pool)
        .await
        .unwrap();
    }
}
