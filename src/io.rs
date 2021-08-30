use anyhow::Result;
use chrono::NaiveDate;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

use crate::plan::Plan;

const LOG_EXT: &str = "plan.md";
const EDITOR: &str = "vim.bat";

#[derive(Debug)]
pub struct PlanFile {
    path: PathBuf,
    plan: Plan,
}

impl PlanFile {
    fn new(path: PathBuf) -> Result<Self> {
        // TODO: validate path exists, is valid
        let doc = fs::read_to_string(path.to_owned())?;
        let plan = Plan::from_markdown(&doc)?;
        Ok(PlanFile { path, plan })
    }

    pub fn edit(&self) {
        log::debug!("Opening plan file for editing: {:#?}", &self.path);

        Command::new(EDITOR)
            .arg(&self.path)
            .status()
            .expect("Editor failed to start.");
    }

    /// Get a reference to the plan from the file.
    pub fn plan(&self) -> &Plan {
        &self.plan
    }

    pub fn create_copy(&self, path: PathBuf, date: NaiveDate) -> anyhow::Result<Self> {
        let mut new_plan = self.plan.clone();
        new_plan.clean();
        new_plan.set_date(date);

        let md = new_plan.to_markdown();
        std::fs::write(path.to_owned(), md)?;

        Ok(Self {
            path,
            plan: new_plan,
        })
    }
}

pub struct PlanDirectory {
    path: PathBuf,
}

impl PlanDirectory {
    pub fn new(path: PathBuf) -> Self {
        log::debug!("Initializing plan directory at '{:#?}'.", path);
        PlanDirectory { path }
    }

    pub fn create_plan(&self, date: NaiveDate) -> anyhow::Result<PlanFile> {
        let plan_path = self.get_plan_path(date);

        log::debug!("Creating plan file for date: {:#?}", date);
        std::fs::File::create(plan_path.to_owned())?;

        PlanFile::new(plan_path)
    }

    /// Creates a clean copy of the provided plan file, and sets the date.
    pub fn copy_plan(&self, original_plan: PlanFile, date: NaiveDate) -> anyhow::Result<PlanFile> {
        let path = self.get_plan_path(date);

        log::debug!(
            "Creating a copy of plan file `{:#?}` for date `{:#?}`",
            original_plan.path,
            date
        );
        original_plan.create_copy(path, date)
    }

    pub fn get_plan(&self, date: NaiveDate) -> Option<PlanFile> {
        let plan_path = self.get_plan_path(date);

        if plan_path.exists() {
            log::trace!("Plan file found at path: {:#?}", plan_path);
            match PlanFile::new(plan_path) {
                Ok(file) => Some(file),
                _ => None,
            }
        } else {
            log::trace!("No plan file found at path: {:#?}", plan_path);
            None
        }
    }

    pub fn get_most_recent_plan(&self) -> Option<PlanFile> {
        let today = chrono::Local::today().naive_local();
        let plan_paths = self.get_files();

        let file = match plan_paths {
            Ok(mut paths) => {
                paths.sort();
                paths.reverse();
                paths
                    .into_iter()
                    .map(PlanFile::new)
                    .flatten()
                    .find(|p| p.plan().date() <= today)
            }
            _ => None,
        };

        log::debug!("Most recent plan: {:#?}", file);
        file
    }

    fn get_plan_path(&self, date: NaiveDate) -> PathBuf {
        let mut plan_path = self.path.to_owned();
        let file_format = format!("%Y.%m.%d.{}", LOG_EXT);
        let today_file_name = date.format(file_format.as_str());

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

    fn is_plan_file(path: &Path) -> bool {
        let is_plan = match path.file_name() {
            Some(file_name) => match file_name.to_str() {
                Some(ext_str) => ext_str.ends_with(LOG_EXT),
                _ => false,
            },
            _ => false,
        };

        log::debug!("{:#?} is plan file: {}", path, is_plan);
        is_plan
    }
}
