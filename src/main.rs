use std::env;
use std::{path::PathBuf, fs};

use anyhow::Result;
use clap::{Parser, Subcommand};
use env_logger;

use rust_runtime::create;
use rust_runtime::start;
use rust_runtime::delete;
use rust_runtime::kill;
use rust_runtime::stop;
use rust_runtime::state;

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
    Start(start::Start),
    Delete(delete::Delete),
    Kill(kill::Kill),
    Stop(stop::Stop),
    State(state::State),
}

impl SubCommand {
    fn get_container_id(&self) -> &String {
        match &self {
            SubCommand::Create(create) => &create.container_id,
            SubCommand::Start(start) => &start.container_id,
            SubCommand::Delete(delete) => &delete.container_id,
            SubCommand::Kill(kill) => &kill.container_id,
            SubCommand::Stop(stop) => &stop.container_id,
            SubCommand::State(state) => &state.container_id,
        }
    }
}

fn main() -> Result<()> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    
    let opts = Opts::parse();
    rust_runtime::logger::init(opts.subcmd.get_container_id().as_str(), opts.log)?;

    let root_path = PathBuf::from(&opts.root);
    fs::create_dir_all(&root_path)?;

    match opts.subcmd {
        SubCommand::Create(create) => create.exec(root_path),
        SubCommand::Start(start) => start.exec(root_path),
        SubCommand::Delete(delete) => delete.exec(root_path),
        SubCommand::Kill(kill) => kill.exec(root_path),
        SubCommand::Stop(stop) => stop.exec(root_path),
        SubCommand::State(state) => state.exec(root_path),
    }
}
