use crate::{NormalizedProgram, NormalizedStatus};
use anyhow::anyhow;

pub fn suggest_next_step(
    cur_program: &str,
    cur_status: &str,
) -> anyhow::Result<(NormalizedProgram, NormalizedStatus)> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_statusmatch_from_asr_to_bestwestern() {
        let (NormalizedProgram { name: program, .. }, NormalizedStatus { name: status, .. }) =
            suggest_next_step("Ascott Star Rewards", "Platinum").unwrap();

        assert_eq!(
            (
                "Best Western Rewards".to_string(),
                "Diamond Select".to_string()
            ),
            (program, status)
        );
    }
}