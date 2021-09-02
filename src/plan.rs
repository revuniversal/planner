mod util;

mod sections;

use std::fmt::Write;

use self::sections::tasks::TaskList;
use self::util::get_node_text;
use crate::plan::sections::tasks::{parse_task_list, TaskStatus};
use chrono::{Datelike, NaiveDate, Weekday};
use comrak::{nodes::NodeValue, parse_document, Arena, ComrakOptions};

use self::sections::schedule::{parse_schedule, Schedule};

#[derive(Debug, Clone)]
pub struct Plan {
    date: NaiveDate,
    tasks: Option<TaskList>,
    schedule: Option<Schedule>,
}
impl Plan {
    #[cfg(test)]
    /// Initializes a new plan.
    fn new(date: NaiveDate, tasks: Option<TaskList>, schedule: Option<Schedule>) -> Self {
        Self {
            date,
            tasks,
            schedule,
        }
    }

    /// Creates a [Plan] from a markdown document.
    pub fn from_markdown(doc: &str) -> anyhow::Result<Self> {
        #[derive(Debug, PartialEq)]
        enum ParseState {
            Initial,
            DateFound,
            TaskSectionStart,
            TaskSectionEnd,
            ScheduleSectionStart,
            ScheduleSectionEnd,
        }

        let mut parse_state = ParseState::Initial;
        let arena = Arena::new();
        let root = parse_document(&arena, doc, &ComrakOptions::default());
        let nodes = root.children();

        let mut tasks = None;
        let mut schedule = None;

        let mut date_str = "".to_string();

        for node in nodes {
            match parse_state {
                ParseState::Initial => {
                    log::trace!("Parsing state: {:#?}", parse_state);
                    let header = match node.data.to_owned().into_inner().value {
                        NodeValue::Heading(c) => c,
                        _ => continue,
                    };

                    if header.level != 1 {
                        continue;
                    }

                    date_str = get_node_text(node);
                    parse_state = ParseState::DateFound;
                    continue;
                }
                ParseState::DateFound => {
                    log::trace!("Parsing state: {:#?}", parse_state);
                    let header = match node.data.to_owned().into_inner().value {
                        NodeValue::Heading(c) => c,
                        _ => continue,
                    };

                    if header.level != 2 {
                        continue;
                    }

                    let header_text = get_node_text(node);

                    if header_text.trim().to_lowercase() == "tasks" {
                        parse_state = ParseState::TaskSectionStart
                    }
                    continue;
                }
                ParseState::TaskSectionStart => {
                    log::trace!("Parsing state: {:#?}", parse_state);
                    match node.data.to_owned().into_inner().value {
                        NodeValue::List(_) => {
                            tasks = Some(parse_task_list(node)?);
                            parse_state = ParseState::TaskSectionEnd;
                        }
                        _ => continue,
                    };
                }
                ParseState::TaskSectionEnd => {
                    log::trace!("Parsing state: {:#?}", parse_state);
                    let header = match node.data.to_owned().into_inner().value {
                        NodeValue::Heading(c) => c,
                        _ => continue,
                    };

                    if header.level != 2 {
                        continue;
                    }

                    let header_text = get_node_text(node);

                    if header_text.trim().to_lowercase() == "schedule" {
                        parse_state = ParseState::ScheduleSectionStart
                    }
                    continue;
                }
                ParseState::ScheduleSectionStart => {
                    log::trace!("Parsing state: {:#?}", parse_state);
                    match node.data.to_owned().into_inner().value {
                        NodeValue::List(_) => {
                            schedule = Some(parse_schedule(node)?);
                            parse_state = ParseState::ScheduleSectionEnd;
                        }
                        _ => continue,
                    };
                }
                ParseState::ScheduleSectionEnd => {
                    log::trace!("Parsing state: {:#?}", parse_state);
                    break;
                }
            }
        }

        let date = chrono::NaiveDate::parse_from_str(date_str.trim(), "%m/%d/%Y")?;

        Ok(Plan {
            date,
            tasks,
            schedule,
        })
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }
    pub fn day(&self) -> &'static str {
        match self.date.weekday() {
            Weekday::Mon => "Monday",
            Weekday::Tue => "Tuseday",
            Weekday::Wed => "Wednesday",
            Weekday::Thu => "Thursday",
            Weekday::Fri => "Friday",
            Weekday::Sat => "Saturday",
            Weekday::Sun => "Sunday",
        }
    }

    pub fn tasks(&self) -> &Option<TaskList> {
        &self.tasks
    }

    /// Get a reference to the plan's schedule.
    pub fn schedule(&self) -> &Option<Schedule> {
        &self.schedule
    }

    /// Exports a copy of the plan as markdown.
    pub fn to_markdown(&self) -> String {
        let mut md: String = String::new();

        writeln!(md, "# {}", self.date().format("%m/%d/%Y")).unwrap();
        writeln!(md, "{}", self.day()).unwrap();
        writeln!(md).unwrap();

        if let Some(tasks) = self.tasks() {
            writeln!(md, "## Tasks").unwrap();
            for category in tasks.categories() {
                writeln!(md, "- **{}**", category.name()).unwrap();
                for task in category.tasks() {
                    match task.status() {
                        TaskStatus::Incomplete => {
                            writeln!(md, "  - [ ] {}", task.description()).unwrap();
                        }
                        TaskStatus::Complete => {}
                    }
                }
            }
            writeln!(md).unwrap();
        }

        if let Some(schedule) = self.schedule() {
            writeln!(md, "## Schedule").unwrap();

            writeln!(md, "- **Planned**").unwrap();
            for event in schedule.planned() {
                let time = event.start().format("%H%M").to_string();
                writeln!(md, "  - {}\t{}", time, event.description()).unwrap();
            }

            writeln!(md, "- **Actual**").unwrap();
            for event in schedule.actual() {
                let time = event.start().format("%H%M").to_string();
                writeln!(md, "  - {}\t{}", time, event.description()).unwrap();
            }
        }
        writeln!(md, "## Notes").unwrap();
        writeln!(md).unwrap();

        md
    }

    /// Sets the plan date.
    pub(crate) fn set_date(&mut self, date: NaiveDate) {
        self.date = date;
    }

    /// Cleans the plan and its subsections.
    pub fn clean(&mut self) {
        log::trace!("Cleaning plan `{:#?}`...", self.date());

        if let Some(tasks) = &mut self.tasks {
            log::trace!("Task section exists. Cleaning...");
            tasks.clean();
            log::trace!("Task section cleaned.");
        }
        if let Some(schedule) = &mut self.schedule {
            log::trace!("Schedule section exists. Cleaning...");
            schedule.clean();
            log::trace!("Schedule section cleaned.");
        }

        log::trace!("Plan cleaned.");
    }
}

#[cfg(test)]
mod tests {
    use crate::plan::sections::tasks::TaskStatus;

    use super::*;
    use indoc::indoc;

    #[test]
    fn minimal_export_md() {
        let date = NaiveDate::from_ymd(2000, 1, 1);
        let plan = Plan::new(date, None, None);
        let md = plan.to_markdown();
        assert_eq!(
            md,
            indoc! {"
                # 01/01/2000
                Saturday

                ## Notes

            "}
        );
    }

    #[test]
    fn minimal_import_tasks() {
        let md = indoc! {"
            # 01/01/2000
            Saturday

            ## Tasks

            - **Personal**
              - [ ] TODO
              - [x] Already done
        "};

        let plan = Plan::from_markdown(md).unwrap();
        let tasks = plan.tasks.unwrap();

        assert_eq!(tasks.categories().len(), 1);

        let personal_tasks = tasks.categories().first().unwrap();
        assert_eq!(personal_tasks.name(), "Personal");
        assert_eq!(personal_tasks.tasks().len(), 2);

        let todo = personal_tasks.tasks().first().unwrap();
        assert_eq!(todo.description(), "TODO");
        assert_eq!(todo.status(), &TaskStatus::Incomplete);

        let already_done = personal_tasks.tasks().last().unwrap();
        assert_eq!(already_done.description(), "Already done");
        assert_eq!(already_done.status(), &TaskStatus::Complete);
    }
}
