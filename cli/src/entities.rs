use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct NormalizedProgram {
    pub id: usize,
    pub name: String,
}

#[derive(Serialize)]
pub struct NormalizedStatus {
    pub id: usize,
    pub program_id: usize,
    pub pos: usize,
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
    pub id: usize,
    pub from_status_id: usize,
    pub to_status_id: usize,
    pub result: ReportResult,
}