use clap::{crate_authors, crate_name, crate_version, AppSettings, Clap};
use std::{env, path::PathBuf};

const DEFAULT_DIR: &str = "~/journal";

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
    /// Gets the directory containing plan files.
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

mod plan_path {
    use chrono::NaiveDate;
    use std::{fs, io, path::PathBuf, process::Command};
    const LOG_EXT: &str = "log";

    pub struct PlanFile {
        path: PathBuf,
    }

    impl PlanFile {
        fn new(path: PathBuf) -> Self {
            // TODO: validate path exists, is valid
            PlanFile { path }
        }

        pub fn get_date(&self) -> NaiveDate {
            let mut path = self.path.to_owned();

            path.set_extension("");
            let date_str = path.file_name().and_then(|e| e.to_str()).unwrap();
            chrono::NaiveDate::parse_from_str(&date_str, "%Y.%m.%d")
                .expect("Could not parse date from filename.")
        }

        pub fn get_path(&self) -> &PathBuf {
            &self.path
        }

        pub fn open(&self) {
            log::debug!("Opening journal file in vim: {:#?}", &self.path);
            Command::new("vim.bat")
                .arg(&self.path)
                .status()
                .expect("vim failed to start.");
        }
    }

    pub struct PlanDirectory {
        path: PathBuf,
    }

    impl PlanDirectory {
        pub fn new(path: PathBuf) -> Self {
            PlanDirectory { path }
        }

        pub fn get_plan(&self, date: NaiveDate) -> PlanFile {
            let mut plan_path = self.path.to_owned();
            let today_file_name = date.format("%Y.%m.%d.log");

            plan_path.push(&today_file_name.to_string());
            PlanFile::new(plan_path)
        }

        pub fn get_most_recent_plan(&self) -> Option<PlanFile> {
            let today = chrono::Local::today().naive_local();
            let plan_paths = self.get_files();
            if plan_paths.is_err() {
                return None;
            } else {
                let mut plan_paths = plan_paths.unwrap();
                plan_paths.sort();
                plan_paths.reverse();

                plan_paths
                    .into_iter()
                    .map(|p| PlanFile::new(p))
                    .find(|p| p.get_date() <= today)
            }
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
}

fn main() -> anyhow::Result<()> {
    use plan_path::*;

    env_logger::init();

    let options = Options::parse();
    let plan_dir = PlanDirectory::new(options.get_root_dir());

    let today = chrono::Local::today().naive_local();
    let today_path = plan_dir.get_plan(today);
    let most_recent_plan = plan_dir.get_most_recent_plan();

    if most_recent_plan.is_some() {}

    let today_file_name = today.format("%Y.%m.%d.log");

    println!("Today name: {}", today_file_name);
    println!(
        "Latest name: {}",
        plan_dir
            .get_most_recent_plan()
            .unwrap()
            .get_path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
    );

    today_path.open();

    Ok(())
}
