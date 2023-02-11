use sqlx::PgPool;

use crate::usecase::UsecaseForMemory;

pub async fn store(db_url: &str, usecase: &UsecaseForMemory) -> anyhow::Result<()> {
    let pool = PgPool::connect(db_url).await?;

    for program in &usecase.programs {
        sqlx::query!(
            "INSERT INTO programs(name) VALUES ($1) ON CONFLICT DO NOTHING",
            program.name
        )
        .execute(&pool)
        .await
        .expect(&format!("{:?}", program));
    }

    for status in &usecase.statuses {
        let program = usecase.find_program_by_id(status.program_id)?;

        sqlx::query!(
            r#"
            INSERT INTO program_statuses(program_id, level, name) 
            VALUES ((SELECT id FROM programs WHERE name = $1), $2, $3)
            ON CONFLICT DO NOTHING
            "#,
            program.name,
            status.level as i32,
            status.name,
        )
        .execute(&pool)
        .await
        .expect(&format!("{:?}", status));
    }

    for report in &usecase.reports {
        let from_status = usecase.find_status_by_id(report.from_status_id)?;
        let from_program = usecase.find_program_by_id(from_status.program_id)?;
        let to_status = usecase.find_status_by_id(report.to_status_id)?;
        let to_program = usecase.find_program_by_id(to_status.program_id)?;
        let result = serde_json::to_string(&report.result)?.replace("\"", "");

        sqlx::query(
            r#"
            INSERT INTO reports (
                from_program_id,
                from_status_level,
                to_program_id,
                to_status_level,
                result
            ) VALUES (
                (SELECT id FROM programs WHERE name = $1),
                $2,
                (SELECT id FROM programs WHERE name = $3),
                $4,
                ($5::report_result)
            )
            "#,
        )
        .bind(&from_program.name)
        .bind(from_status.level as i32)
        .bind(&to_program.name)
        .bind(to_status.level as i32)
        .bind(result)
        .execute(&pool)
        .await
        .expect(&format!("{:?}", report));
    }
    Ok(())
}
