use crate::entities::*;
use crate::Usecase;

pub fn suggest_next_step<'a>(
    usecase: &'a dyn Usecase,
    cur_program: &str,
    cur_status: &str,
) -> anyhow::Result<Vec<(&'a NormalizedProgram, &'a NormalizedStatus)>> {
    usecase.suggest_next_step(cur_program, cur_status)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usecase::UsecaseForMemory;
    use anyhow::anyhow;

    impl Usecase for UsecaseForMemory {
        fn suggest_next_step(
            &self,
            cur_program: &str,
            cur_status: &str,
        ) -> anyhow::Result<Vec<(&NormalizedProgram, &NormalizedStatus)>> {
            let program = self
                .programs
                .iter()
                .find(|p| p.name == cur_program)
                .ok_or(anyhow!("the program is not found."))?;
            let status = self
                .statuses
                .iter()
                .find(|s| s.program_id == program.id && s.name == cur_status)
                .ok_or(anyhow!("the status is not found."))?;

            let reports = self
                .reports
                .iter()
                .filter(move |r| r.from_status_id == status.id);

            let result = reports
                .map(|r| {
                    let to_status = self
                        .statuses
                        .iter()
                        .find(|s| s.id == r.to_status_id)
                        .unwrap();
                    let to_program = self
                        .programs
                        .iter()
                        .find(|p| p.id == to_status.program_id)
                        .unwrap();
                    (to_program, to_status)
                })
                .collect();

            Ok(result)
        }
    }

    fn create_program_and_statuses(
        program_id: usize,
        program: &str,
        statuses: &[(usize, &str)],
    ) -> (NormalizedProgram, Vec<NormalizedStatus>) {
        let program = NormalizedProgram {
            id: program_id,
            name: program.into(),
        };
        let statuses = statuses
            .iter()
            .enumerate()
            .map(|(pos, (id, name))| NormalizedStatus {
                id: *id,
                pos,
                name: name.to_string(),
                program_id,
            })
            .collect();
        (program, statuses)
    }

    fn create_report(id: usize, from_status_id: usize, to_status_id: usize) -> NormalizedReport {
        NormalizedReport {
            id,
            from_status_id,
            to_status_id,
            result: ReportResult::MATCH,
        }
    }

    fn create_usecase() -> UsecaseForMemory {
        let (asr, asr_statuses) =
            create_program_and_statuses(83822, "Ascott Star Rewards", &[(83826, "Platinum")]);
        let (bestwestern, bestwestern_statuses) = create_program_and_statuses(
            21170,
            "Best Western Rewards",
            &[(551150, "Diamond Select")],
        );
        let (ihg, ihg_statuses) =
            create_program_and_statuses(21207, "IHG One Rewards", &[(35289, "Platinum Elite")]);
        let (mariott, mariott_statuses) =
            create_program_and_statuses(21221, "Marriott Bonvoy", &[(22740, "Gold Elite")]);

        let asr_to_bestwestern_report = create_report(0, 83826, 551150);

        let ihg_marriott_report = create_report(1, 35289, 22740);

        UsecaseForMemory {
            programs: vec![asr, bestwestern, ihg, mariott],
            statuses: vec![
                asr_statuses,
                bestwestern_statuses,
                ihg_statuses,
                mariott_statuses,
            ]
            .into_iter()
            .flatten()
            .collect(),
            reports: vec![asr_to_bestwestern_report, ihg_marriott_report],
        }
    }

    #[test]
    fn should_be_able_to_statusmatch_from_asr_to_bestwestern() {
        let usecase = create_usecase();
        if let [(NormalizedProgram { name: program, .. }, NormalizedStatus { name: status, .. }), ..] =
            suggest_next_step(&usecase, "Ascott Star Rewards", "Platinum").unwrap()[..]
        {
            assert_eq!(
                ("Best Western Rewards", "Diamond Select"),
                (program.as_str(), status.as_str())
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn should_be_able_to_statusmatch_from_ihg_to_marriott() {
        let usecase = create_usecase();
        if let [(NormalizedProgram { name: program, .. }, NormalizedStatus { name: status, .. }), ..] =
            suggest_next_step(&usecase, "IHG One Rewards", "Platinum Elite").unwrap()[..]
        {
            assert_eq!(
                ("Marriott Bonvoy", "Gold Elite"),
                (program.as_str(), status.as_str())
            );
        } else {
            assert!(false);
        }
    }
}
