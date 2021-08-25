use crate::plan::util::get_node_text;
use chrono::{Datelike, NaiveDate, NaiveTime, Weekday};
use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, ComrakOptions,
};

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
enum ScheduleSection {
    Planned,
    Actual,
}

#[derive(Debug)]
pub struct Event {
    description: String,
    start: NaiveTime,
}
impl Event {
    /// Get a reference to the event's description.
    pub fn description(&self) -> &str {
        self.description.as_str()
    }

    /// Get a reference to the event's start.
    pub fn start(&self) -> &NaiveTime {
        &self.start
    }
}

#[derive(Debug)]
pub struct Schedule {
    planned: Vec<Event>,
    actual: Vec<Event>,
}

impl Schedule {
    /// Get a reference to the planned events.
    pub fn planned(&self) -> &[Event] {
        self.planned.as_slice()
    }

    /// Get a reference to the actual events.
    pub fn actual(&self) -> &[Event] {
        self.actual.as_slice()
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

fn parse_event<'a>(node: &'a AstNode<'a>) -> Option<Event> {
    log::debug!("Parsing event");

    match &node.data.borrow().value {
        NodeValue::Item(_) => {
            let text = get_node_text(node);
            let mut pieces = text.split('\t').filter(|&x| !x.is_empty());

            let time = pieces.next().and_then(|time_str| {
                log::trace!("Parsing time from string '{}'...", time_str);
                match NaiveTime::parse_from_str(time_str, "%H%M") {
                    Ok(t) => {
                        log::trace!("Parsed time: '{:#?}'", t);
                        Some(t)
                    }
                    _ => {
                        log::trace!("Could not parse time");
                        None
                    }
                }
            });
            let description = pieces.next().map(|x| x.to_owned());

            match (time, description) {
                (Some(start), Some(description)) => Some(Event { start, description }),
                (Some(start), None) => {
                    log::warn!(
                        "Parsed time {:#?}, but could not parse description from {}",
                        start,
                        text
                    );
                    None
                }
                _ => {
                    log::warn!("Could not parse event string '{}'", text);
                    None
                }
            }
        }
        _ => {
            log::warn!(
                "Expected NodeValue::Item, but found {:#?}",
                &node.data.borrow().value
            );

            None
        }
    }
}

fn parse_schedule<'a>(node: &'a AstNode<'a>) -> anyhow::Result<Schedule> {
    log::debug!("Parsing schedule...");

    let mut current_section: ScheduleSection = ScheduleSection::Planned;
    let mut planned: Vec<Event> = Vec::new();
    let mut actual: Vec<Event> = Vec::new();

    for ul_child in node.children() {
        match &ul_child.data.borrow().value {
            NodeValue::Item(_) => {
                for li_child in ul_child.children() {
                    match &li_child.data.borrow().value {
                        NodeValue::Paragraph => {
                            let text = get_node_text(li_child);
                            if text.trim().to_lowercase() == "planned" {
                                log::trace!("Parsing planned events");
                                current_section = ScheduleSection::Planned;
                            } else if text.trim().to_lowercase() == "actual" {
                                log::trace!("Parsing actual events");
                                current_section = ScheduleSection::Actual;
                            } else {
                                log::error!(
                                    "Schedule parsing error:  Expected 'Planned' or 'Actual', but found '{}'",
                                    text
                                );
                                return Err(anyhow::anyhow!(
                                    "Expected 'Planned' or 'Actual', but found '{}'",
                                    text
                                ));
                            }
                        }
                        NodeValue::List(_) => {
                            for inner_ul_child in li_child.children() {
                                if let Some(event) = parse_event(inner_ul_child) {
                                    match current_section {
                                        ScheduleSection::Planned => planned.push(event),
                                        ScheduleSection::Actual => actual.push(event),
                                    }
                                }
                            }
                        }
                        _ => log::warn!(
                            "Expected NodeValue::List or NodeValue::Paragraph, but found {:#?}",
                            &li_child.data.borrow().value
                        ),
                    }
                }
            }
            _ => log::warn!(
                "Expected NodeValue::Item, but found {:#?}",
                &ul_child.data.borrow().value
            ),
        };
    }

    Ok(Schedule { planned, actual })
}

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
