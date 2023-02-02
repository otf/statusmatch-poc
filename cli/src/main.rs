use std::env;

use anyhow::bail;
use cli::{scrape, usecase::{UsecaseForMemory, Usecase}};

fn main() -> anyhow::Result<()> {
    let data = scrape::run()?;
    let usecase = UsecaseForMemory::load_from(data);

    if let [_, cur_program, cur_status] = &env::args().collect::<Vec<_>>()[..] {
        let next_steps = usecase.suggest_next_step(&cur_program, &cur_status)?;

        println!("Your next step:");
        for (next_program, next_status) in next_steps {
            println!("* {}({})", next_program.name, next_status.name);
        }
        Ok(())
    } else {
        bail!("[example] cli 'IHG One Rewards' 'Platinum Elite'")
    }
}
