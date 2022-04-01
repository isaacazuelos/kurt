//! Kurt - A language for fun

use clap::{Parser, Subcommand};

mod eval;
mod repl;
mod script;

use eval::Evaluate;
use repl::ReplArgs;
use script::Script;

#[derive(clap::Parser)]
#[clap(name = "kurt")]
#[clap(subcommand_required = true)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Script(Script),
    Repl(ReplArgs),
    Eval(Evaluate),
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Some(Command::Script(script)) => script.run(),
        Some(Command::Repl(_args)) => {
            panic!("repl broken, reloading needs work")
        }
        Some(Command::Eval(eval)) => eval.run(),

        None => unreachable!("arg parser should print help"),
    }
}
