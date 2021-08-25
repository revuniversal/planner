use chrono::NaiveTime;
use comrak::nodes::{AstNode, NodeValue};

use crate::plan::util::get_node_text;

#[derive(Debug)]
pub enum ScheduleSection {
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

pub fn parse_schedule<'a>(node: &'a AstNode<'a>) -> anyhow::Result<Schedule> {
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
