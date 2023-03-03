use std::env;

use dormys::retrieve_status;

fn main() -> anyhow::Result<()> {
    let args: Vec<_> = env::args().collect();
    let email = &args[1];
    let password = &args[2];
    let status = retrieve_status(email, password)?;
    println!("{}", status);
    Ok(())
}
