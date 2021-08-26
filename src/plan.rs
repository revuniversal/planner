mod schedule;
mod tasks;
mod util;
use std::fmt::Write;

use crate::plan::tasks::parse_task_list;

use self::{tasks::TaskList, util::get_node_text};
use chrono::{Datelike, NaiveDate, Weekday};
use comrak::{nodes::NodeValue, parse_document, Arena, ComrakOptions};

use self::schedule::{parse_schedule, Schedule};

#[derive(Debug)]
pub struct Plan {
    date: NaiveDate,
    tasks: Option<TaskList>,
    schedule: Option<Schedule>,
}
impl Plan {
    /// Creates a [Plan] from a markdown document.
    pub fn from_document(doc: &str) -> anyhow::Result<Self> {
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

    pub fn to_markdown(&self) -> String {
        let mut md: String = String::new();

        writeln!(md, "## {}", self.date().format("%m/%d/%Y")).unwrap();
        writeln!(md, "{}", self.day()).unwrap();
        writeln!(md).unwrap();

        if let Some(tasks) = self.tasks() {
            writeln!(md, "## Tasks:").unwrap();
            for category in tasks.categories() {
                writeln!(md, "- **{}**", category.name()).unwrap();
                for task in category.tasks() {
                    match task.status() {
                        self::tasks::TaskStatus::Incomplete => {
                            writeln!(md, "  - [ ] {}", task.description()).unwrap();
                        }
                        self::tasks::TaskStatus::Complete => {}
                    }
                }
            }
            writeln!(md).unwrap();
        }

        if let Some(schedule) = self.schedule() {
            writeln!(md, "## Schedule:").unwrap();

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

        writeln!(md).unwrap();

        md
    }
}
