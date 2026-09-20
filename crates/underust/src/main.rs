//! The underust CLI.

mod cmd;
mod repo;
mod toolchain;

use clap::{Parser, Subcommand};
use underust_core::manifest::Mode;

#[derive(Parser)]
#[command(name = "underust", version, about = "A Rust internals gym")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List tasks.
    List {
        /// Only tasks whose track starts with this prefix.
        #[arg(long)]
        track: Option<String>,
        /// Only tasks in this mode.
        #[arg(long, value_parser = parse_mode)]
        mode: Option<Mode>,
    },
    /// Check every manifest for schema and consistency errors.
    Validate,
    /// Report what this machine can grade, and how to fix what it cannot.
    Doctor,
}

fn parse_mode(raw: &str) -> Result<Mode, String> {
    match raw {
        "build" => Ok(Mode::Build),
        "predict" => Ok(Mode::Predict),
        "optimize" => Ok(Mode::Optimize),
        "review" => Ok(Mode::Review),
        "design" => Ok(Mode::Design),
        "constrain" => Ok(Mode::Constrain),
        "soundness" => Ok(Mode::Soundness),
        other => Err(format!("unknown mode `{other}`")),
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let root = repo::root()?;
    match cli.command {
        Command::List { track, mode } => cmd::list::run(&root, track.as_deref(), mode),
        Command::Doctor => cmd::doctor::run(&root),
        Command::Validate => {
            let count = cmd::validate::run(&root)?;
            println!("validate: {count} manifests ok");
            Ok(())
        }
    }
}
