mod cli;
mod io;
mod plan;

use clap::Clap;
use comrak::{parse_document, Arena, ComrakOptions};

use crate::cli::Options;
use crate::io::*;
use crate::plan::TaskStatus;

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
        cli::Command::Ast => {
            let arena = Arena::new();
            let md = today_plan.content()?;
            let root = parse_document(&arena, &md, &ComrakOptions::default());

            println!("{:#?}", root);
        }
        cli::Command::View => {
            let md = today_plan.content()?;
            let plan = plan::Plan::from_document(&md)?;

            println!("## {}", plan.date().format("%m/%d/%Y"));
            println!("{}", plan.day());
            println!();

            if let Some(tasks) = plan.tasks() {
                println!("## Tasks:");
                for category in tasks.categories() {
                    println!("- **{}**", category.name());
                    for task in category.tasks() {
                        match task.status() {
                            TaskStatus::Incomplete => println!("  - [ ] {}", task.description()),
                            TaskStatus::Complete => {}
                        }
                    }
                }
                println!();
            }

            if let Some(schedule) = plan.schedule() {
                println!("## Schedule:");
                println!("- **Planned**");
                for event in schedule.planned() {
                    let time = event.start().format("%H%M").to_string();
                    println!("  - {}\t{}", time, event.description());
                }
                println!("- **Actual**");
                for event in schedule.actual() {
                    let time = event.start().format("%H%M").to_string();
                    println!("  - {}\t{}", time, event.description());
                }
                println!();
            }
        }
        cli::Command::Edit => {
            today_plan.edit();
        }
    }

    Ok(())
}
