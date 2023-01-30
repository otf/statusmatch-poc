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

#[derive(Serialize)]
struct NormalizedStatus {
    program_id: usize,
    pos: usize,
    name: String,
}

fn retrieve_program_and_statuses() -> reqwest::Result<Vec<ProgramAndStatus>> {
    let res = reqwest::blocking::get(
        "https://www.statusmatcher.com/api/program?view=programAndStatuses",
    )?
    .json::<Vec<ProgramAndStatus>>()?;
    Ok(res)
}

fn dump(path: &str, data: &impl Serialize) -> anyhow::Result<()> {
    let json = serde_json::to_string(&data)?;
    let mut file = File::create(path)?;
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

fn normalize_statuses(
    programs: &Vec<NormalizedProgram>,
    program_and_statuses: &Vec<ProgramAndStatus>,
) -> Vec<NormalizedStatus> {
    programs
        .iter()
        .flat_map(|program| {
            program_and_statuses
                .iter()
                .find(|row| row.name == program.name)
                .unwrap()
                .statuses
                .iter()
                .enumerate()
                .map(|(i, row)| NormalizedStatus {
                    program_id: program.id,
                    pos: i,
                    name: row.name.clone(),
                })
        })
        .collect()
}

fn main() -> anyhow::Result<()> {
    let program_and_statuses = retrieve_program_and_statuses()?;
    let programs = normalize_programs(&program_and_statuses);
    dump("programs.json", &programs)?;
    let statuses = normalize_statuses(&programs, &program_and_statuses);
    dump("statuses.json", &statuses)?;
    Ok(())
}
