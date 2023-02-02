use crate::entities::*;

pub trait Usecase {
    fn suggest_next_step(
        &self,
        cur_program: &str,
        cur_status: &str,
    ) -> anyhow::Result<Vec<(&NormalizedProgram, &NormalizedStatus)>>;
}

pub struct UsecaseForMemory {
    pub programs: Vec<NormalizedProgram>,
    pub statuses: Vec<NormalizedStatus>,
    pub reports: Vec<NormalizedReport>,
}
