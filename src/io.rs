
use chrono::NaiveDate;
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};
const LOG_EXT: &str = "plan.md";

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
        // need to remove both .plan and .md, so strip the extension twice
        let mut path = self.path.to_owned();
        path.set_extension("");
        path.set_extension("");

        let date_str = path.file_name().and_then(|e| e.to_str()).unwrap();
        chrono::NaiveDate::parse_from_str(date_str, "%Y.%m.%d")
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
        log::debug!("Initializing plan directory at '{:#?}'.", path);
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
            log::debug!("No plan file found at path: {:#?}", plan_path);
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
                    .find(|p| p.get_date() <= today)
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
