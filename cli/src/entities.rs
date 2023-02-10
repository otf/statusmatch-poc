use serde::{Deserialize, Serialize};

pub type Entities = (
    Vec<NormalizedProgram>,
    Vec<NormalizedStatus>,
    Vec<NormalizedReport>,
);

#[derive(Serialize, Deserialize)]
pub struct NormalizedProgram {
    pub id: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct NormalizedStatus {
    pub id: usize,
    pub program_id: usize,
    pub level: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum ReportResult {
    #[serde(rename(serialize = "match"))]
    MATCH,
    #[serde(rename(serialize = "deny"))]
    DENY,
    #[serde(rename(serialize = "challenge"))]
    CHALLENGE,
}

#[derive(Serialize, Deserialize)]
pub struct NormalizedReport {
    pub id: usize,
    pub from_status_id: usize,
    pub to_status_id: usize,
    pub result: ReportResult,
}
