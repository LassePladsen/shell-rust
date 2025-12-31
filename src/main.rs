use rustyline::Result;

mod command;
mod env;
mod file;
mod input;
mod repl;

fn main() -> Result<()> {
    repl::start_repl()?;
    Ok(())
}
