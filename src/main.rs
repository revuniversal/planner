mod cli;

use clap::Clap;
use cli::Options;

mod plan_path {
    use chrono::NaiveDate;
    use std::{fs, io, path::PathBuf, process::Command};
    const LOG_EXT: &str = "log";

    #[derive(Debug)]
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

        pub fn edit(&self) {
            log::debug!("Opening plan file in vim: {:#?}", &self.path);

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

        pub fn create_plan(&self, date: NaiveDate) -> anyhow::Result<PlanFile> {
            let plan_path = self.get_plan_path(date);

            log::debug!("Creating plan file for date: {:#?}", date);
            std::fs::File::create(plan_path.to_owned())?;

            Ok(PlanFile::new(plan_path))
        }

        pub fn copy_plan(&self, plan_to_copy: PlanFile, date: NaiveDate) -> io::Result<PlanFile> {
            let path = self.get_plan_path(date);

            log::debug!("Copying plan file: {:#?} to {:#?}", plan_to_copy, path);
            std::fs::copy(plan_to_copy.path, path.to_owned())?;

            Ok(PlanFile::new(path))
        }

        pub fn get_plan(&self, date: NaiveDate) -> Option<PlanFile> {
            let plan_path = self.get_plan_path(date);

            if plan_path.exists() {
                log::debug!("Plan file found at path: {:#?}", plan_path);
                Some(PlanFile::new(plan_path))
            } else {
                log::debug!("NO plan file found at path: {:#?}", plan_path);
                None
            }
        }

        pub fn get_most_recent_plan(&self) -> Option<PlanFile> {
            let today = chrono::Local::today().naive_local();
            let plan_paths = self.get_files();

            match plan_paths {
                Ok(mut paths) => {
                    paths.sort();
                    paths.reverse();
                    paths
                        .into_iter()
                        .map(|p| PlanFile::new(p))
                        .find(|p| p.get_date() <= today)
                }
                _ => None,
            }
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
}

fn main() -> anyhow::Result<()> {
    use plan_path::*;

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

    today_plan.edit();

    Ok(())
}
