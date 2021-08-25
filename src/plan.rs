use self::util::get_node_text;
use chrono::{Datelike, NaiveDate, Weekday};
use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, ComrakOptions,
};

use self::schedule::{parse_schedule, Schedule};

#[derive(Debug)]
pub struct TaskList {
    categories: Vec<TaskCategory>,
}
impl TaskList {
    pub fn categories(&self) -> &Vec<TaskCategory> {
        &self.categories
    }
}

#[derive(Debug)]
pub enum TaskStatus {
    Complete,
    Incomplete,
}

#[derive(Debug)]
pub struct Task {
    description: String,
    status: TaskStatus,
}
impl Task {
    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn status(&self) -> &TaskStatus {
        &self.status
    }
}

#[derive(Debug)]
pub struct TaskCategory {
    name: String,
    tasks: Vec<Task>,
}
impl TaskCategory {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tasks(&self) -> &Vec<Task> {
        &self.tasks
    }
}
#[derive(Debug)]
pub struct Plan {
    date: NaiveDate,
    tasks: Option<TaskList>,
    schedule: Option<Schedule>,
}
impl Plan {
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
}

mod schedule;

fn parse_task_list<'a>(node: &'a AstNode<'a>) -> anyhow::Result<TaskList> {
    let mut categories: Vec<TaskCategory> = Vec::new();

    for child in node.children() {
        categories.push(parse_task_category(child)?);
    }

    Ok(TaskList { categories })
}

fn parse_task_category<'a>(node: &'a AstNode<'a>) -> anyhow::Result<TaskCategory> {
    let mut tasks: Vec<Task> = Vec::new();
    let mut name: String = "".to_string();

    for node in node.children() {
        match &node.data.borrow().value {
            NodeValue::Paragraph => name = name + &get_node_text(node),
            NodeValue::List(_) => {
                for node in node.children() {
                    if let NodeValue::Item(_) = &node.data.borrow().value {
                        let text = get_node_text(node);
                        let status = if text.starts_with("[x]") {
                            TaskStatus::Complete
                        } else {
                            TaskStatus::Incomplete
                        };

                        tasks.push(Task {
                            description: text
                                .trim_start_matches("[ ]")
                                .trim_start_matches("[x]")
                                .trim()
                                .to_string(),
                            status,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    Ok(TaskCategory { name, tasks })
}

mod util;
