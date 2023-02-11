use serde::{Deserialize, Serialize};

pub type Entities = (
    Vec<NormalizedProgram>,
    Vec<NormalizedStatus>,
    Vec<NormalizedReport>,
);

#[derive(Serialize, Deserialize, Debug)]
pub struct NormalizedProgram {
    pub id: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NormalizedStatus {
    pub id: usize,
    pub program_id: usize,
    pub level: usize,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "report_result")]
#[sqlx(rename_all = "lowercase")]
pub enum NormalizedReportResult {
    #[serde(rename = "match")]
    MATCH,
    #[serde(rename = "deny")]
    DENY,
    #[serde(rename = "challenge")]
    CHALLENGE,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NormalizedReport {
    pub id: usize,
    pub from_status_id: usize,
    pub to_status_id: usize,
    pub result: NormalizedReportResult,
}
