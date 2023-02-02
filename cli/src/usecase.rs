use crate::entities::*;
use anyhow::anyhow;
use itertools::Itertools;

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

impl UsecaseForMemory {
    pub fn load_from((programs, statuses, reports): Entities) -> Self {
        Self {
            programs: programs,
            statuses: statuses,
            reports: reports,
        }
    }
    fn find_program_by_name(&self, program: &str) -> anyhow::Result<&NormalizedProgram> {
            self.programs
            .iter()
            .find(|p| p.name.to_lowercase().contains(&program.to_lowercase()))
            .ok_or(anyhow!("the program is not found."))
    }

    fn find_status_by_name(&self, program: &NormalizedProgram, status: &str) -> anyhow::Result<&NormalizedStatus> {
            self.statuses
            .iter()
            .find(|s| s.program_id == program.id && s.name.to_lowercase() == status.to_lowercase())
            .ok_or(anyhow!("the status is not found."))
    }

    fn find_status_by_id(&self, status_id: usize) -> anyhow::Result<&NormalizedStatus> {
        self
            .statuses
            .iter()
            .find(|s| s.id == status_id)
            .ok_or(anyhow!("the status is not found."))
    }

    fn find_program_by_id(&self, program_id: usize) -> anyhow::Result<&NormalizedProgram> {
        self
            .programs
            .iter()
            .find(|p| p.id == program_id)
            .ok_or(anyhow!("the program is not found."))
    }
}

impl Usecase for UsecaseForMemory {
    fn suggest_next_step(
        &self,
        cur_program: &str,
        cur_status: &str,
    ) -> anyhow::Result<Vec<(&NormalizedProgram, &NormalizedStatus)>> {
        let program = self.find_program_by_name(cur_program)?;
        let status = self.find_status_by_name(program, cur_status)?;

        let reports = self
            .reports
            .iter()
            .filter(move |r| r.from_status_id == status.id)
            .sorted_by(|a,b| {
                let to_status_a = self.find_status_by_id(a.to_status_id).unwrap();
                let to_status_b = self.find_status_by_id(b.to_status_id).unwrap();

                to_status_b.pos.cmp(&to_status_a.pos)
            })
            .unique_by(|r| (r.from_status_id, r.to_status_id));

        let result = reports
            .map(|r| {
                let to_status = self.find_status_by_id(r.to_status_id).unwrap();
                let to_program = self.find_program_by_id(to_status.program_id).unwrap();
                (to_program, to_status)
            })
            .collect();

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    fn create_program_and_statuses(
        program_id: usize,
        program: &str,
        statuses: Vec<(usize, &str)>,
    ) -> (NormalizedProgram, Vec<NormalizedStatus>) {
        let program = NormalizedProgram {
            id: program_id,
            name: program.into(),
        };
        let statuses = statuses
            .into_iter()
            .enumerate()
            .map(|(pos, (id, name))| NormalizedStatus {
                id: id,
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
            create_program_and_statuses(83822, "Ascott Star Rewards", vec![(83826, "Platinum")]);
        let (bestwestern, bestwestern_statuses) = create_program_and_statuses(
            21170,
            "Best Western Rewards",
            vec![(551150, "Diamond Select")],
        );
        let (ihg, ihg_statuses) =
            create_program_and_statuses(21207, "IHG One Rewards", vec![(35289, "Platinum Elite")]);
        let (mariott, mariott_statuses) =
            create_program_and_statuses(21221, "Marriott Bonvoy", vec![(22742, "Silver Elite"), (22740, "Gold Elite")]);

        let asr_to_bestwestern_report = create_report(0, 83826, 551150);

        let ihg_marriott_report = create_report(1, 35289, 22740);

        let ihg_marriott_report_dup = create_report(2, 35289, 22742);

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
            reports: vec![asr_to_bestwestern_report, ihg_marriott_report_dup, ihg_marriott_report],
        }
    }

    #[test_case(("Ascott Star Rewards", "Platinum"), ("Best Western Rewards", "Diamond Select"))]
    #[test_case(("IHG One Rewards", "Platinum Elite"), ("Marriott Bonvoy", "Gold Elite"); "Duplicated report has added.")]
    #[test_case(("ihg", "platinum elite"), ("Marriott Bonvoy", "Gold Elite"); "Ambiguous input.")]
    fn should_be_able_to_suggest((from_program, from_status): (&str, &str), (to_program, to_status): (&str, &str)) {
        let usecase = create_usecase();
        if let [(NormalizedProgram { name: program, .. }, NormalizedStatus { name: status, .. }), ..] =
            usecase.suggest_next_step(from_program, from_status).unwrap()[..]
        {
            assert_eq!(
                (to_program, to_status),
                (program.as_str(), status.as_str())
            );
        } else {
            assert!(false);
        }
    }
}