use std::path::PathBuf;

use clap::{crate_authors, crate_name, crate_version, AppSettings, Clap};

// NOTE: These 2 constants should be changed together.  Not worth the time to
// fix.  Deal with it.
const PLANNER_DIR: &str = ".planner";

/// A plaintext planning tool for a particular kind of nerd.  
#[derive(Clap, Debug)]
#[clap(name = crate_name!())]
#[clap(version = crate_version!())]
#[clap(author = crate_authors!())]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Options {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Clap, Debug)]
pub enum Command {
    /// Edit the plan in Vim.
    Edit,
    /// Send the plan to STDOUT.
    View,
}

impl Options {
    /// Gets the directory containing plan files.
    pub fn get_root_dir(&self) -> PathBuf {
        let mut home = dirs::home_dir().expect("Could not open home directory.");
        home.push(PLANNER_DIR);
        home
    }

    pub fn command(self) -> Command {
        match self.command {
            Some(c) => c,
            _ => Command::Edit,
        }
    }
}
