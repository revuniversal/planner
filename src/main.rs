mod cli;
mod io;
mod plan;

use clap::Clap;

use crate::cli::Options;
use crate::io::*;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = Options::parse();

    let plan_dir = PlanDirectory::new(options.get_root_dir());
    let today = chrono::Local::today().naive_local();
    let today_plan = plan_dir.get_plan(today);

    let today_plan = match today_plan {
        Some(p) => p,
        None => {
            let most_recent_plan = plan_dir.get_most_recent_plan();
            match most_recent_plan {
                Some(p) => plan_dir.copy_plan(p, today)?,
                None => plan_dir.create_plan(today)?,
            }
        }
    };

    match options.command() {
        cli::Command::View => {
            let parsed = today_plan.plan().to_markdown();
            print!("{}", parsed);
        }
        cli::Command::Edit => {
            today_plan.edit();
        }
    }

    Ok(())
}
