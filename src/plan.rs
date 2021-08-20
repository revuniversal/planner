use std::cell::RefCell;

use chrono::{Datelike, NaiveDate, ParseError, Weekday};
use comrak::{
    arena_tree::Children,
    nodes::{Ast, AstNode, NodeValue},
    parse_document, Arena, ComrakOptions,
};

pub struct Plan {
    date: NaiveDate,
    day: Weekday,
}

impl Plan {
    pub fn from_document(doc: &str) -> anyhow::Result<Self> {
        let arena = Arena::new();
        let root = parse_document(&arena, doc, &ComrakOptions::default());
        let nodes = root.children();
        let date = parse_date(nodes)?;
        let day = date.weekday();

        Ok(Plan { date, day })
    }

    pub fn date(&self) -> NaiveDate {
        self.date
    }
    pub fn day(&self) -> Weekday {
        self.day
    }
}

pub fn parse_date(nodes: Children<RefCell<Ast>>) -> Result<NaiveDate, ParseError> {
    let mut date_str = "".to_string();

    for node in nodes {
        let header = match node.data.to_owned().into_inner().value {
            NodeValue::Heading(c) => c,
            _ => continue,
        };

        if header.level != 1 {
            continue;
        }

        let mut text = Vec::new();
        collect_text(node, &mut text);

        date_str = String::from_utf8(text).unwrap();
    }

    chrono::NaiveDate::parse_from_str(&date_str.trim(), "%m/%d/%Y")
}

fn collect_text<'a>(node: &'a AstNode<'a>, output: &mut Vec<u8>) {
    match node.data.borrow().value {
        NodeValue::Text(ref literal) => output.extend_from_slice(literal),
        NodeValue::LineBreak | NodeValue::SoftBreak => output.push(b' '),
        _ => {
            for n in node.children() {
                collect_text(n, output);
            }
        }
    }
}
