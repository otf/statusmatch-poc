use crate::entities::*;
use itertools::Itertools;
use reqwest::blocking as req;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::create_dir_all;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

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

#[derive(Deserialize, Debug)]
struct ReportList {
    collection: Vec<Report>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
struct Report {
    id: usize,
    result: ReportResult,
    from_program: Option<String>,
    from_status: Option<String>,
    to_program: Option<String>,
    to_status: Option<String>,
}

fn retrieve_program_and_statuses() -> reqwest::Result<Vec<ProgramAndStatus>> {
    let url = "https://www.statusmatcher.com/api/program?view=programAndStatuses";
    Ok(req::get(url)?.json()?)
}

fn retrieve_reports(to_program: &NormalizedProgram) -> reqwest::Result<Vec<Report>> {
    let url = format!("https://www.statusmatcher.com/api/report?page=0&size={}&view=programReportList&programId={}&to=true", u16::MAX, to_program.id);
    Ok(req::get(url)?.json::<ReportList>()?.collection)
}

fn accumulate_reports(programs: &Vec<NormalizedProgram>) -> reqwest::Result<Vec<Report>> {
    Ok(programs
        .iter()
        .filter_map(|row| retrieve_reports(row).ok())
        .flatten()
        .collect())
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
        .map(|row| NormalizedProgram {
            id: row.id,
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
                .find(|row| row.id == program.id)
                .unwrap()
                .statuses
                .iter()
                .enumerate()
                .map(|(i, row)| NormalizedStatus {
                    id: row.id,
                    program_id: program.id,
                    pos: i,
                    name: row.name.clone(),
                })
        })
        .collect()
}

fn find_status_id(
    programs: &Vec<NormalizedProgram>,
    statuses: &Vec<NormalizedStatus>,
    program: &Option<String>,
    status: &Option<String>,
) -> Option<usize> {
    if let (Some(program), Some(status)) = (program, status) {
        let program = programs.iter().find(|row| row.name == *program).unwrap();
        let status = statuses
            .iter()
            .find(|row| row.program_id == program.id && row.name == *status)
            .unwrap();
        Some(status.id)
    } else {
        None
    }
}

fn normalize_reports(
    programs: &Vec<NormalizedProgram>,
    statuses: &Vec<NormalizedStatus>,
    reports: &Vec<Report>,
) -> Vec<NormalizedReport> {
    reports
        .iter()
        .filter_map(|report| {
            if let (Some(from_status_id), Some(to_status_id)) = (
                find_status_id(
                    programs,
                    statuses,
                    &report.from_program,
                    &report.from_status,
                ),
                find_status_id(programs, statuses, &report.to_program, &report.to_status),
            ) {
                Some(NormalizedReport {
                    id: report.id,
                    from_status_id: from_status_id,
                    to_status_id: to_status_id,
                    result: report.result,
                })
            } else {
                None
            }
        })
        .collect()
}

fn create_dir_if_not_exists(path: &str) -> std::io::Result<bool> {
    let path = Path::new(path);
    if path.try_exists()? {
        Ok(true)
    } else {
        create_dir_all(path)?;
        Ok(false)
    }
}

fn load<T: DeserializeOwned>(path: &str) -> io::Result<Vec<T>> {
    Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
}

const PROGRAMS_PATH: &str = "data/programs.json";
const STATUSES_PATH: &str = "data/statuses.json";
const REPORTS_PATH: &str = "data/reports.json";

pub fn run() -> anyhow::Result<Entities> {
    let normalized_programs;
    let normalized_statuses;
    let normalized_reports;

    if !create_dir_if_not_exists("data")? {
        let program_and_statuses = retrieve_program_and_statuses()?;
        normalized_programs = normalize_programs(&program_and_statuses);
        normalized_statuses = normalize_statuses(&normalized_programs, &program_and_statuses);
        let reports = accumulate_reports(&normalized_programs)?;
        normalized_reports =
            normalize_reports(&normalized_programs, &normalized_statuses, &reports);

        dump(PROGRAMS_PATH, &normalized_programs)?;
        dump(STATUSES_PATH, &normalized_statuses)?;
        dump(REPORTS_PATH, &normalized_reports)?;
    } else {
        normalized_programs = load(PROGRAMS_PATH)?;
        normalized_statuses = load(STATUSES_PATH)?;
        normalized_reports = load(REPORTS_PATH)?;
    }

    Ok((normalized_programs, normalized_statuses, normalized_reports))
}
