use pulldown_cmark::{Event, Parser, Tag, TagEnd};

pub struct MarkdownContent {
    pub raw: String,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub text: String,
    pub url: String,
    pub start: usize,
    pub end: usize,
}

pub fn parse(content: &str) -> MarkdownContent {
    let parser = Parser::new(content);
    let mut links = Vec::new();
    let mut current_text = String::new();
    let mut in_link = false;
    let mut link_text = String::new();
    let mut link_url = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Link { dest_url, .. }) => {
                in_link = true;
                link_url = dest_url.to_string();
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                in_link = false;
                links.push(Link {
                    text: link_text.clone(),
                    url: link_url.clone(),
                    start: current_text.len(),
                    end: current_text.len() + link_text.len(),
                });
                current_text.push_str(&link_text);
            }
            Event::Text(text) => {
                if in_link {
                    link_text.push_str(&text);
                }
                current_text.push_str(&text);
            }
            _ => {}
        }
    }

    MarkdownContent {
        raw: content.to_string(),
        links,
    }
}
