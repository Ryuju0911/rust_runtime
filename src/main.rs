use std::{path::PathBuf, fs};

use anyhow::Result;
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

impl SubCommand {
    fn get_container_id(&self) -> &String {
        match &self {
            SubCommand::Create(create) => &create.container_id,
        }
    }
}

fn main() -> Result<()> {
    let opts = Opts::parse();
    rust_runtime::logger::init(opts.subcmd.get_container_id().as_str(), opts.log)?;

    let root_path = PathBuf::from(&opts.root);
    fs::create_dir_all(&root_path)?;
    match opts.subcmd {
        SubCommand::Create(create) => create.exec(root_path)
    }
}
