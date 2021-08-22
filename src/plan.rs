use chrono::{Datelike, NaiveDate, Weekday};
use comrak::{
    nodes::{AstNode, NodeValue},
    parse_document, Arena, ComrakOptions,
};

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
pub struct TaskList {
    categories: Vec<TaskCategory>,
}
impl TaskList {
    pub fn categories(&self) -> &Vec<TaskCategory> {
        &self.categories
    }
}

#[derive(Debug)]
pub struct Plan {
    date: NaiveDate,
    tasks: TaskList,
}
impl Plan {
    pub fn from_document(doc: &str) -> anyhow::Result<Self> {
        #[derive(PartialEq)]
        enum ParseState {
            Initial,
            DateFound,
            TaskSectionStart,
            TaskSectionEnd,
        }

        let mut parse_state = ParseState::Initial;
        let arena = Arena::new();
        let root = parse_document(&arena, doc, &ComrakOptions::default());
        let nodes = root.children();

        let mut tasks = TaskList {
            categories: Vec::new(),
        };
        let mut date_str = "".to_string();

        for node in nodes {
            match parse_state {
                ParseState::Initial => {
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
                    match node.data.to_owned().into_inner().value {
                        NodeValue::List(_) => {
                            tasks = parse_task_list(node)?;
                            parse_state = ParseState::TaskSectionEnd;
                        }
                        _ => continue,
                    };
                }
                ParseState::TaskSectionEnd => {}
            }
        }

        let date = chrono::NaiveDate::parse_from_str(date_str.trim(), "%m/%d/%Y")?;

        Ok(Plan { date, tasks })
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

    pub fn tasks(&self) -> &TaskList {
        &self.tasks
    }
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

fn get_node_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut text_bytes = Vec::new();
    collect_text(node, &mut text_bytes);
    String::from_utf8(text_bytes).unwrap().trim().to_string()
}

/// Collect the text and
fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) => output.extend_from_slice(literal),
        NodeValue::Code(ref literal) => {
            output.push(b'`');
            output.extend_from_slice(literal);
            output.push(b'`');
        }
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}
