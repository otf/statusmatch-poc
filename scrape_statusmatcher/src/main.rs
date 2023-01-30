use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Status {
    id: usize,
    name: String,
}

#[derive(Serialize, Deserialize)]
struct ProgramAndStatus {
    id: usize,
    name: String,
    statuses: Vec<Status>,
}

#[derive(Serialize)]
struct NormalizedProgram {
    id: usize,
    name: String,
}

fn retrieve_program_and_statuses() -> reqwest::Result<Vec<ProgramAndStatus>> {
    let res = reqwest::blocking::get(
        "https://www.statusmatcher.com/api/program?view=programAndStatuses",
    )?
    .json::<Vec<ProgramAndStatus>>()?;
    Ok(res)
}

fn dump_programs(programs: &Vec<NormalizedProgram>) -> anyhow::Result<()> {
    let json = serde_json::to_string(&programs)?;
    let mut file = File::create("./program.json")?;
    file.write_all(json.as_bytes())?;
    file.flush()?;
    Ok(())
}

fn normalize_programs(program_and_statuses: &Vec<ProgramAndStatus>) -> Vec<NormalizedProgram> {
    program_and_statuses
        .iter()
        .unique_by(|row| &row.name)
        .enumerate()
        .map(|(i, row)| NormalizedProgram {
            id: i,
            name: row.name.clone(),
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let program_and_statuses = retrieve_program_and_statuses()?;
    let programs = normalize_programs(&program_and_statuses);
    dump_programs(&programs)?;
    Ok(())
}
