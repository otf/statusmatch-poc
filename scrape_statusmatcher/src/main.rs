use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Status {
    id: i64,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct ProgramAndStatuses {
    id: i64,
    name: String,
    statuses: Vec<Status>,
}

fn retrieve_programs() -> reqwest::Result<Vec<ProgramAndStatuses>> {
    let res = reqwest::blocking::get(
        "https://www.statusmatcher.com/api/program?view=programAndStatuses",
    )?
    .json::<Vec<ProgramAndStatuses>>()?;
    Ok(res)
}

fn dump_programs(program_and_statuses: &Vec<ProgramAndStatuses>) -> anyhow::Result<()> {
    let programs = program_and_statuses
        .iter()
        .unique_by(|x| &x.name)
        .enumerate()
        .map(|(i, row)| {
            json!({
                "id": i,
                "name": row.name,
            })
        })
        .collect::<Vec<_>>();
    let json = json!(programs);
    let str_json = serde_json::to_string(&json)?;
    let mut file = File::create("./program.json")?;
    file.write_all(str_json.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let program_and_statuses = retrieve_programs()?;
    dump_programs(&program_and_statuses)?;
    Ok(())
}
