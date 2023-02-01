use crate::{NormalizedProgram, NormalizedStatus};

pub trait Usecase {
    fn suggest_next_step(
        &self,
        cur_program: &str,
        cur_status: &str,
    ) -> anyhow::Result<Vec<(&NormalizedProgram, &NormalizedStatus)>>;
}
