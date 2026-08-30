use super::model::Policy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

impl ParseError {
    fn new(line: usize, message: impl Into<String>) -> Self {
        Self {
            line,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Filesystem,
    Process,
    Network,
}

pub fn parse(input: &str) -> Result<Policy, ParseError> {
    let policy = Policy::default();
    let mut section: Option<Section> = None;

    for (index, raw_line) in input.lines().enumerate() {
        let line_number = index + 1;

        // Remove comments.
        let line = raw_line.split('#').next().unwrap_or("").trim();

        // Ignore blank lines.
        if line.is_empty() {
            continue;
        }

        // Parse section headers.
        if line.starts_with('[') && line.ends_with(']') {
            section = Some(match line {
                "[filesystem]" => Section::Filesystem,
                "[process]" => Section::Process,
                "[network]" => Section::Network,
                _ => {
                    return Err(ParseError::new(
                        line_number,
                        format!("unknown section: {line}"),
                    ));
                }
            });

            continue;
        }

        // Rules aren't implemented yet.
        if section.is_none() {
            return Err(ParseError::new(line_number, "rule found before a section"));
        }

        return Err(ParseError::new(line_number, "rules are not supported yet"));
    }

    Ok(policy)
}
