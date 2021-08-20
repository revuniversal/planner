mod cli;
mod io;

use clap::Clap;
use cli::Options;
use comrak::{parse_document, Arena, ComrakOptions};
use io::*;

mod plan {
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
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let options = Options::parse();

    let plan_dir = PlanDirectory::new(options.get_root_dir());
    let today = chrono::Local::today().naive_local();
    let today_plan = plan_dir.get_plan(today);

    let today_plan = match today_plan {
        Some(p) => p,
        None => {
            let most_recent_plan = plan_dir.get_most_recent_plan();
            match most_recent_plan {
                Some(p) => plan_dir.copy_plan(p, today)?,
                None => plan_dir.create_plan(today)?,
            }
        }
    };

    match options.command() {
        cli::Command::Ast => {
            let arena = Arena::new();
            let md = today_plan.content()?;
            let root = parse_document(&arena, &md, &ComrakOptions::default());

            println!("{:#?}", root);
        }
        cli::Command::View => {
            let md = today_plan.content()?;
            let plan = plan::Plan::from_document(&md)?;

            println!("Date: {}", plan.date().format(format!("%m/%d/%Y").as_str()));
            println!("Day: {:?}", plan.day());
        }
        cli::Command::Edit => {
            today_plan.edit();
        }
    }

    Ok(())
}
