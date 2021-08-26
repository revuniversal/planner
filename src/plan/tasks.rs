use comrak::nodes::{AstNode, NodeValue};

use super::util::get_node_text;

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

pub fn parse_task_list<'a>(node: &'a AstNode<'a>) -> anyhow::Result<TaskList> {
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
