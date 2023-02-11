use std::env;

use anyhow::bail;
use cli::{
    db, scrape,
    usecase::{Usecase, UsecaseForMemory},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data = scrape::run()?;
    let usecase = UsecaseForMemory::load_from(data);

    let args = &env::args().collect::<Vec<_>>()[..];
    if let [_, cur_program, cur_status] = args {
        let next_steps = usecase.suggest_next_step(&cur_program, &cur_status)?;

        println!("Your next step:");
        for (next_program, next_status) in next_steps {
            println!("* {}({})", next_program.name, next_status.name);
        }
        Ok(())
    } else if let [_, db_url] = args {
        std::env::set_var("DATABASE_URL", db_url);
        db::store(db_url, &usecase).await?;
        Ok(())
    } else {
        bail!("[example] cli 'IHG One Rewards' 'Platinum Elite'")
    }
}
