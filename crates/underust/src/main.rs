//! The underust CLI.

mod cargo;
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
    /// Rung one: a nudge. Always available.
    Hint {
        /// The task id.
        id: String,
    },
    /// Rung two: the full solution. Refuses while the task's tests fail.
    ///
    /// The seal is obfuscation, not secrecy. This gate is policy: it makes the easy
    /// path the honest one, and does not pretend to stop a determined reader.
    Reveal {
        /// The task id.
        id: String,
        /// Override the gate. Recorded in local progress.
        #[arg(long)]
        stuck: bool,
    },
    /// Show what has been solved, hinted and revealed on this machine.
    Progress,
    /// Gate 3: every task stub compiles.
    CiStubs,
    /// Gate 4: every sealed solution unseals and passes its own tests.
    Prove,
    /// Gate 6: every radar invalidation names a task that exists.
    RadarCheck,
    /// Author-side: seal a solution directory into .sealed/<id>.seal.
    Seal {
        /// The task id.
        id: String,
        /// Directory holding EXPLANATION.md and files/.
        #[arg(long)]
        from: std::path::PathBuf,
    },
    /// Scaffold a blank prediction for a predict task.
    Predict {
        /// The task id.
        id: String,
    },
    /// Grade one task.
    Test {
        /// The task id, for example drop/01-field-order.
        id: String,
        /// Run inside the canonical linux image instead of on this host.
        #[arg(long)]
        docker: bool,
    },
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
        Command::Hint { id } => cmd::reveal::hint(&root, &id),
        Command::Reveal { id, stuck } => cmd::reveal::reveal(&root, &id, stuck),
        Command::Progress => cmd::reveal::show(&root),
        Command::CiStubs => cmd::gates::ci_stubs(&root).map(|_| ()),
        Command::Prove => cmd::gates::prove(&root).map(|_| ()),
        Command::RadarCheck => cmd::gates::radar_check(&root).map(|_| ()),
        Command::Seal { id, from } => cmd::seal_tool::run(&root, &id, &from),
        Command::Predict { id } => {
            let path = cmd::predict::scaffold(&root, &id)?;
            println!("wrote {}", path.display());
            println!("fill it in, then run: underust test {id}");
            Ok(())
        }
        Command::Test { id, docker } => {
            if cmd::test::run(&root, &id, docker)? {
                Ok(())
            } else {
                std::process::exit(1)
            }
        }
        Command::Validate => {
            let count = cmd::validate::run(&root)?;
            println!("validate: {count} manifests ok");
            Ok(())
        }
    }
}
