pub mod scrape;
pub mod statusmatch;
pub mod usecase;

pub use usecase::Usecase;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct NormalizedProgram {
    id: usize,
    pub name: String,
}

#[derive(Serialize)]
pub struct NormalizedStatus {
    id: usize,
    program_id: usize,
    pos: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ReportResult {
    MATCH,
    DENY,
    CHALLENGE,
}

#[derive(Serialize, Debug)]
pub struct NormalizedReport {
    id: usize,
    from_status_id: usize,
    to_status_id: usize,
    result: ReportResult,
}
