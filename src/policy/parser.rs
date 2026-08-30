use super::model::{HostPattern, PathPattern, Policy};

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

fn parse_values<'a>(
    parts: impl Iterator<Item = &'a str>,
    line_number: usize,
    error: &str,
) -> Result<Vec<String>, ParseError> {
    let values = parts.collect::<Vec<_>>().join(" ");

    let values = values
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if values.is_empty() {
        return Err(ParseError::new(line_number, error));
    }

    Ok(values)
}

fn parse_filesystem_rule(
    line: &str,
    policy: &mut Policy,
    line_number: usize,
) -> Result<(), ParseError> {
    let mut parts = line.split_whitespace();

    let action = parts
        .next()
        .ok_or_else(|| ParseError::new(line_number, "missing action"))?;

    let operation = parts
        .next()
        .ok_or_else(|| ParseError::new(line_number, "missing filesystem operation"))?;

    let values = parse_values(parts, line_number, "missing filesystem path")?;

    let rules = match action {
        "allow" => &mut policy.filesystem.allow,
        "ask" => &mut policy.filesystem.ask,
        "deny" => &mut policy.filesystem.deny,
        _ => {
            return Err(ParseError::new(
                line_number,
                format!("unknown action: {action}"),
            ));
        }
    };

    let patterns = values.into_iter().map(PathPattern).collect::<Vec<_>>();

    match operation {
        "read" => rules.read.extend(patterns),
        "write" => rules.write.extend(patterns),
        "delete" => rules.delete.extend(patterns),
        _ => {
            return Err(ParseError::new(
                line_number,
                format!("unknown filesystem operation: {operation}"),
            ));
        }
    }

    Ok(())
}

fn parse_process_rule(
    line: &str,
    policy: &mut Policy,
    line_number: usize,
) -> Result<(), ParseError> {
    let mut parts = line.split_whitespace();

    let action = parts
        .next()
        .ok_or_else(|| ParseError::new(line_number, "missing action"))?;

    let rule = parts
        .next()
        .ok_or_else(|| ParseError::new(line_number, "missing process rule"))?;

    if rule == "scope" {
        if action != "allow" {
            return Err(ParseError::new(line_number, "scope can only use allow"));
        }

        let scopes = parse_values(parts, line_number, "missing process scope")?;

        policy
            .process
            .scope
            .extend(scopes.into_iter().map(PathPattern));

        return Ok(());
    }

    if rule != "command" {
        return Err(ParseError::new(
            line_number,
            format!("unknown process rule: {rule}"),
        ));
    }

    let commands = parse_values(parts, line_number, "missing process command")?;

    let rules = match action {
        "allow" => &mut policy.process.allow,
        "ask" => &mut policy.process.ask,
        "deny" => &mut policy.process.deny,
        _ => {
            return Err(ParseError::new(
                line_number,
                format!("unknown process action: {action}"),
            ));
        }
    };

    rules.extend(commands);

    Ok(())
}

fn parse_network_rule(
    line: &str,
    policy: &mut Policy,
    line_number: usize,
) -> Result<(), ParseError> {
    let mut parts = line.split_whitespace();

    let action = parts
        .next()
        .ok_or_else(|| ParseError::new(line_number, "missing action"))?;

    let rule = parts
        .next()
        .ok_or_else(|| ParseError::new(line_number, "missing network rule"))?;

    if rule != "host" {
        return Err(ParseError::new(
            line_number,
            format!("unknown network rule: {rule}"),
        ));
    }

    let hosts = parse_values(parts, line_number, "missing network host")?;

    let rules = match action {
        "allow" => &mut policy.network.allow,
        "ask" => &mut policy.network.ask,
        "deny" => &mut policy.network.deny,
        _ => {
            return Err(ParseError::new(
                line_number,
                format!("unknown network action: {action}"),
            ));
        }
    };

    rules.extend(hosts.into_iter().map(HostPattern));

    Ok(())
}

pub fn parse(input: &str) -> Result<Policy, ParseError> {
    let mut policy = Policy::default();
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
        match section {
            Some(Section::Filesystem) => {
                parse_filesystem_rule(line, &mut policy, line_number)?;
            }
            Some(Section::Process) => {
                parse_process_rule(line, &mut policy, line_number)?;
            }
            Some(Section::Network) => {
                parse_network_rule(line, &mut policy, line_number)?;
            }
            None => {
                return Err(ParseError::new(line_number, "rule found before a section"));
            }
        }
    }

    Ok(policy)
}
