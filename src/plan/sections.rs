use chrono::SecondsFormat;
use comrak::nodes::{AstNode, NodeValue};

use super::util::get_node_text;

pub(super) mod schedule;
pub(super) mod tasks;

/**
Contains references to the nodes that comprise a section of the plan.

A plan begins with a level 2 header (`##`), and ends at the beginning of the
next section, or the end of the document.

## Example markdown

```markdown
## Miscellaneous

This is just some miscellaneous text in a miscellaneous section.

### `<h3>` elements don't signify the beginning of a new section.

#### Neither do `<h4>` elements.

##### Nor `<h5>`.

###### Nor `<h6>`.

####### There's no such thing as `<h7>`!

- This is a miscellaneous list item.
- And another one.

## Another section begins at this `<h2>` element.

## And this is yet another section, even if the previous one is empty.

```

*/
struct PlanSection<'a> {
    /// The AST node that represents the section heading.
    heading: &'a AstNode<'a>,

    /// The AST nodes that represent the contents of the section.
    contents: Vec<&'a AstNode<'a>>,
}

impl<'a> PlanSection<'a> {
    fn from_node(node: &'a AstNode<'a>) -> Option<Self> {
        let mut heading_node: &AstNode<'a>;
        let mut contents: Vec<&'a AstNode> = Vec::new();

        match node.data.to_owned().into_inner().value {
            NodeValue::Heading(h) => {
                if h.level != 2 {
                    return None;
                } else {
                    heading_node = node;
                }
            }
            _ => return None,
        }

        for node in heading_node.following_siblings().skip(1) {
            if let NodeValue::Heading(h) = node.data.to_owned().into_inner().value {
                if h.level <= 2 {
                    break;
                }
            }

            contents.push(node);
        }

        Some(PlanSection {
            heading: heading_node,
            contents,
        })
    }

    pub fn get_text(&self) -> String {
        self.contents
            .iter()
            .map(|&n| get_node_text(n))
            .collect::<Vec<String>>()
            .join(" ")
    }
}

fn parse_sections<'a>(root: &'a AstNode<'a>) -> impl Iterator<Item = PlanSection> {
    root.children()
        .filter_map(PlanSection::from_node)
        .into_iter()
}

#[cfg(test)]
mod Tests {
    use crate::plan::util::get_node_text;

    use super::*;
    use comrak::{parse_document, Arena, ComrakOptions};
    use indoc::indoc;

    const TEST_DOC: &str = indoc! {"
        ## Section 1

        ## Section 2

        This is some text here.

    "};

    #[test]
    fn section_length_is_accurate() {
        let arena = Arena::new();
        let root = parse_document(&arena, TEST_DOC, &ComrakOptions::default());
        let sections = parse_sections(root);

        assert_eq!(sections.count(), 2)
    }

    #[test]
    fn section_headings_are_accurate() {
        let arena = Arena::new();
        let root = parse_document(&arena, TEST_DOC, &ComrakOptions::default());
        let sections = parse_sections(root).collect::<Vec<_>>();

        let section_1 = sections.first().unwrap();
        let section_2 = sections.last().unwrap();

        assert_eq!(get_node_text(section_1.heading), "Section 1");
        assert_eq!(get_node_text(section_2.heading), "Section 2");
    }

    #[test]
    fn section_text_is_accurate() {
        let arena = Arena::new();
        let root = parse_document(&arena, TEST_DOC, &ComrakOptions::default());
        let sections = parse_sections(root).collect::<Vec<PlanSection>>();

        let section_1 = sections.first().unwrap();
        let section_2 = sections.last().unwrap();

        let text_1 = section_1.get_text();
        let text_2 = section_2.get_text();

        assert_eq!(text_1, "");
        assert_eq!(text_2, "This is some text here.");
    }
}
