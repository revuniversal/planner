use chrono::NaiveDate;
use clap::{crate_authors, crate_name, crate_version, AppSettings, Clap};
use std::{env, fs, io, path::PathBuf, process::Command};

const DEFAULT_DIR: &str = "~/journal";
const LOG_EXT: &str = "log";

/// A plaintext planning tool for a particular kind of nerd.
#[derive(Clap)]
#[clap(name = crate_name!())]
#[clap(version = crate_version!())]
#[clap(author = crate_authors!())]
#[clap(setting = AppSettings::ColoredHelp)]
struct Options {
    /// The directory that contains the journal files.
    #[clap(long, default_value = DEFAULT_DIR)]
    path: String,
}

impl Options {
    fn get_root_dir(self) -> PathBuf {
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

struct Journal {
    path: PathBuf,
}

impl Journal {
    fn new(path: PathBuf) -> Self {
        Journal { path }
    }

    fn open_in_vim(journal_file: PathBuf) {
        log::debug!("Opening journal file in vim: {:#?}", journal_file);
        Command::new("vim.bat")
            .arg(journal_file)
            .status()
            .expect("vim failed to start.");
    }

    fn get_plan_path(&self, date: NaiveDate) -> PathBuf {
        let mut plan_path = self.path.to_owned();
        let today_file_name = date.format("%Y.%m.%d.log");

        plan_path.push(&today_file_name.to_string());
        plan_path
    }

    fn get_files(&self) -> io::Result<Vec<PathBuf>> {
        Ok(fs::read_dir(&self.path)?
            .map(|res| res.map(|entry| entry.path()))
            .filter(|e| e.is_ok())
            .map(|e| e.unwrap())
            .filter(|e| Self::is_plan_file(e))
            .collect::<Vec<PathBuf>>())
    }

    fn is_plan_file(path: &PathBuf) -> bool {
        match path.extension() {
            Some(ext) => match ext.to_str() {
                Some(ext_str) => ext_str == LOG_EXT,
                _ => false,
            },
            _ => false,
        }
    }
}

fn main() -> io::Result<()> {
    env_logger::init();

    let options = Options::parse();
    let journal_dir = Journal::new(options.get_root_dir());

    let today = chrono::Local::today().naive_local();
    let today_path = journal_dir.get_plan_path(today);

    let mut all_files = journal_dir.get_files()?;
    all_files.sort();

    let latest_file = all_files.last();

    let latest_file_date = match latest_file {
        Some(some_path) => {
            let mut path = some_path.to_owned();
            path.set_extension("");
            let date_str = path
                .file_name()
                .and_then(|e| e.to_str())
                .map(|e| e.replace(".", "-"));
            Some(chrono::NaiveDate::parse_from_str(
                &date_str.unwrap(),
                "%Y-%m-%d",
            ))
        }

        _ => None,
    };

    let today_file_name = today.format("%Y.%m.%d.log");

    println!("Today name: {}", today_file_name);
    println!(
        "Latest name: {}",
        latest_file.unwrap().file_name().unwrap().to_str().unwrap()
    );

    Journal::open_in_vim(today_path);

    Ok(())
}
