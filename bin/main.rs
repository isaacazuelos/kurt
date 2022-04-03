//! Kurt - A language for fun

use clap::{Parser, Subcommand};

mod eval;
mod repl;
mod script;

use eval::Evaluate;
use repl::Repl;
use script::Script;

#[derive(clap::Parser)]
#[clap(author, version, about, name = "kurt")]
#[clap(subcommand_required = true)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,

    /// Enable tracing of the VM
    #[clap(short, long)]
    trace: bool,
}

#[derive(Subcommand)]
enum Command {
    Script(Script),
    Repl(Repl),
    Eval(Evaluate),
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Some(Command::Script(script)) => script.run(&args),
        Some(Command::Repl(repl)) => repl.run(&args),
        Some(Command::Eval(eval)) => eval.run(&args),

        None => unreachable!("arg parser should print help"),
    }
}
