//! Kurt - A language for fun

use clap::{Parser, Subcommand};

mod eval;
mod script;

use eval::Evaluate;
use script::Script;

#[derive(clap::Parser)]
#[clap(author, version, about, name = "kurt")]
#[clap(subcommand_required = true)]
struct Args {
    #[clap(subcommand)]
    command: Option<Command>,

    /// Display the compiled code instead of running it
    #[clap(short, long)]
    dump: bool,
}

#[derive(Subcommand)]
enum Command {
    Script(Script),
    Eval(Evaluate),
}

fn main() {
    let args = Args::parse();

    match &args.command {
        Some(Command::Script(script)) => script.run(&args),
        Some(Command::Eval(eval)) => eval.run(&args),

        None => unreachable!("arg parser should print help"),
    }
}
