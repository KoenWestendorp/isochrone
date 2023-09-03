type Url = String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Line {
    // Just text.
    Text(String),
    // =>[<whitespace>]<URL>[<whitespace><USER-FRIENDLY LINK NAME>]
    Link {
        url: Url,
        name: Option<String>,
    },
    // ```<ALT>
    // <CONTENT>+
    // ```<_DISCARDED>
    Pre {
        alt: Option<String>,
        content: String,
    },
    // #[#[#]][<whitespace>]<CONTENT>
    Heading {
        level: u8,
        content: String,
    },
    // * <TEXT>
    ListItem(String),
    // ><TEXT>
    Quote(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page {
    lines: Vec<Line>,
}

impl Page {
    pub fn parse(text: &str) -> Self {
        let mut lines = Vec::new();

        let mut raw_lines = text.lines();
        while let Some(line) = raw_lines.next() {
            // Link.
            if line.starts_with("=>") {
                let trailing = line[2..].trim_start();
                let (url, name) =
                    if let Some((url, name)) = trailing.split_once(char::is_whitespace) {
                        (url.to_string(), Some(name.trim().to_string()))
                    } else {
                        (trailing.trim().to_string(), None)
                    };
                lines.push(Line::Link { url, name });
                continue;
            }

            // Heading.
            if line.starts_with("#") {
                let level = line[..3].chars().filter(|&ch| ch == '#').count();
                let content = line[level..].trim().to_string();
                lines.push(Line::Heading {
                    level: level as u8,
                    content,
                });
                continue;
            }

            // List item.
            if line.starts_with("* ") {
                let trailing = line[2..].trim();
                lines.push(Line::ListItem(trailing.to_string()));
                continue;
            }

            // Quote.
            if line.starts_with(">") {
                let trailing = line[1..].trim();
                lines.push(Line::Quote(trailing.to_string()));
                continue;
            }

            // Preformatted text.
            if line.starts_with("```") {
                let alt = {
                    let trailing = &line[3..].trim();
                    if trailing.is_empty() {
                        None
                    } else {
                        Some(trailing.to_string())
                    }
                };
                let mut content = String::new();
                while let Some(line) = raw_lines.next() {
                    if line.starts_with("```") {
                        break;
                    }
                    content.push_str(line);
                }
                lines.push(Line::Pre { alt, content });
                continue;
            }

            // Turns out it is just a plain text line.
            lines.push(Line::Text(line.trim().to_string()))
        }

        Self { lines }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let text = "
# Example Title

Welcome to my Gemini capsule.

* Example list item

=> gemini://link.to/another/resource Link text


These are some paragraphs.
They come right after each other.";
        let page = Page::parse(text);

        let expected = Page {
            lines: vec![
                Line::Text("".to_string()),
                Line::Heading {
                    level: 1,
                    content: "Example Title".to_string(),
                },
                Line::Text("".to_string()),
                Line::Text("Welcome to my Gemini capsule.".to_string()),
                Line::Text("".to_string()),
                Line::ListItem("Example list item".to_string()),
                Line::Text("".to_string()),
                Line::Link {
                    url: "gemini://link.to/another/resource".to_string(),
                    name: Some("Link text".to_string()),
                },
                Line::Text("".to_string()),
                Line::Text("".to_string()),
                Line::Text("These are some paragraphs.".to_string()),
                Line::Text("They come right after each other.".to_string()),
            ],
        };

        assert_eq!(page, expected)
    }

    #[test]
    fn links() {
        let text = [
            "=> gemini://example.org/",
            "=> gemini://example.org/ An example link",
            "=> gemini://example.org/foo	Another example link at the same host",
            "=> foo/bar/baz.txt	A relative link",
            "=> 	gopher://example.org:70/1 A gopher link",
        ]
        .join("\n");
        let page = Page::parse(&text);

        let expected = Page {
            lines: vec![
                Line::Link {
                    url: "gemini://example.org/".to_string(),
                    name: None,
                },
                Line::Link {
                    url: "gemini://example.org/".to_string(),
                    name: Some("An example link".to_string()),
                },
                Line::Link {
                    url: "gemini://example.org/foo".to_string(),
                    name: Some("Another example link at the same host".to_string()),
                },
                Line::Link {
                    url: "foo/bar/baz.txt".to_string(),
                    name: Some("A relative link".to_string()),
                },
                Line::Link {
                    url: "gopher://example.org:70/1".to_string(),
                    name: Some("A gopher link".to_string()),
                },
            ],
        };

        assert_eq!(page, expected)
    }
}
