use std::path::PathBuf;

use clap::{crate_authors, crate_name, crate_version, AppSettings, Clap};

const DEFAULT_DIR: &str = "~/journal";

/// A plaintext planning tool for a particular kind of nerd.  #[derive(Clap)]
#[clap(name = crate_name!())]
#[clap(version = crate_version!())]
#[clap(author = crate_authors!())]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Options {
    /// The directory that contains the plan files.
    #[clap(long, default_value = DEFAULT_DIR)]
    pub path: String,
}

impl Options {
    /// Gets the directory containing plan files.
    pub fn get_root_dir(self) -> PathBuf {
        if self.path != DEFAULT_DIR {
            PathBuf::from(self.path)
        } else {
            match dirs::home_dir() {
                Some(mut dir) => {
                    dir.push("journal");
                    dir
                }
                None => PathBuf::from(self.path),
            }
        }
    }
}
