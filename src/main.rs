use std::path::PathBuf;

use clap::{Parser, Subcommand};

use rust_runtime::create;

#[derive(Parser, Debug)]
struct Opts {
    #[clap(short, long, default_value = "/run/youki")]
    root: PathBuf,
    #[clap(short, long)]
    log: Option<PathBuf>,
    #[clap(long)]
    log_format: Option<String>,
    #[clap(subcommand)]
    subcmd: SubCommand, 
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Create(create::Create),
}

fn main() {
    let opts = Opts::parse();
    println!("Input command is {:?}", opts.subcmd)
}
