//! Parser for the scripting DSL

use super::Command;
use anyhow::Result;
use std::collections::HashMap;

/// Parses a script into a sequence of commands
pub fn parse_script(input: &str) -> Result<Vec<Command>> {
    let mut commands = Vec::new();
    let mut lines = input.lines();

    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if let Some(command) = parse_command(line, &mut lines)? {
            commands.push(command);
        }
    }

    Ok(commands)
}

fn parse_command<'a>(
    line: &str,
    lines: &mut impl Iterator<Item = &'a str>,
) -> Result<Option<Command>> {
    let tokens: Vec<_> = line.split_whitespace().collect();
    if tokens.is_empty() {
        return Ok(None);
    }

    match tokens[0] {
        "apply_pattern" => {
            if tokens.len() < 3 {
                anyhow::bail!("apply_pattern requires name and target");
            }
            let name = tokens[1].to_string();
            let target = tokens[2].to_string();
            let options = parse_block(lines)?;

            Ok(Some(Command::ApplyPattern {
                name,
                target,
                options,
            }))
        }

        "analyze" => {
            if tokens.len() < 2 {
                anyhow::bail!("analyze requires target");
            }
            let target = tokens[1].to_string();
            let (patterns, options) = parse_analysis_block(lines)?;

            Ok(Some(Command::Analyze {
                target,
                patterns,
                options,
            }))
        }

        "generate" => {
            if tokens.len() < 3 {
                anyhow::bail!("generate requires name and kind");
            }
            let name = tokens[1].to_string();
            let kind = tokens[2].to_string();
            let (fields, options) = parse_generation_block(lines)?;

            Ok(Some(Command::Generate {
                name,
                kind,
                fields,
                options,
            }))
        }

        "configure" => {
            if tokens.len() < 2 {
                anyhow::bail!("configure requires section name");
            }
            let section = tokens[1].to_string();
            let settings = parse_block(lines)?;

            Ok(Some(Command::Configure { section, settings }))
        }

        _ => {
            anyhow::bail!("Unknown command: {}", tokens[0]);
        }
    }
}

fn parse_block(lines: &mut impl Iterator<Item = &str>) -> Result<HashMap<String, String>> {
    let mut options = HashMap::new();

    // Expect opening brace
    if let Some(line) = lines.next() {
        let line = line.trim();
        if line != "{" {
            anyhow::bail!("Expected opening brace, found: {}", line);
        }
    }

    // Parse block content
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line == "}" {
            break;
        }

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        let parts: Vec<_> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid option format: {}", line);
        }

        let key = parts[0].trim().to_string();
        let value = parts[1].trim().trim_matches('"').to_string();
        options.insert(key, value);
    }

    Ok(options)
}

fn parse_analysis_block(
    lines: &mut impl Iterator<Item = &str>,
) -> Result<(Vec<String>, HashMap<String, bool>)> {
    let mut patterns = Vec::new();
    let mut options = HashMap::new();

    // Expect opening brace
    if let Some(line) = lines.next() {
        let line = line.trim();
        if line != "{" {
            anyhow::bail!("Expected opening brace, found: {}", line);
        }
    }

    // Parse block content
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line == "}" {
            break;
        }

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if line.starts_with("detect ") {
            patterns.push(line[7..].trim().to_string());
        } else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_lowercase() == "true";
            options.insert(key, value);
        }
    }

    Ok((patterns, options))
}

fn parse_generation_block(
    lines: &mut impl Iterator<Item = &str>,
) -> Result<(Vec<String>, HashMap<String, String>)> {
    let mut fields = Vec::new();
    let mut options = HashMap::new();

    // Expect opening brace
    if let Some(line) = lines.next() {
        let line = line.trim();
        if line != "{" {
            anyhow::bail!("Expected opening brace, found: {}", line);
        }
    }

    // Parse block content
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line == "}" {
            break;
        }

        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        if line.starts_with("field ") {
            fields.push(line[6..].trim().to_string());
        } else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            options.insert(key, value);
        }
    }

    Ok((fields, options))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pattern_command() -> Result<()> {
        let script = r#"
            apply_pattern builder MyStruct {
                async = "true"
                visibility = "pub"
            }
        "#;

        let commands = parse_script(script)?;
        assert_eq!(commands.len(), 1);

        if let Command::ApplyPattern {
            name,
            target,
            options,
        } = &commands[0]
        {
            assert_eq!(name, "builder");
            assert_eq!(target, "MyStruct");
            assert_eq!(options.get("async").unwrap(), "true");
            assert_eq!(options.get("visibility").unwrap(), "pub");
        } else {
            panic!("Expected ApplyPattern command");
        }

        Ok(())
    }

    #[test]
    fn test_parse_analyze_command() -> Result<()> {
        let script = r#"
            analyze src/lib.rs {
                detect unsafe_blocks
                detect blocking_io
                check_safety = true
            }
        "#;

        let commands = parse_script(script)?;
        assert_eq!(commands.len(), 1);

        if let Command::Analyze {
            target,
            patterns,
            options,
        } = &commands[0]
        {
            assert_eq!(target, "src/lib.rs");
            assert!(patterns.contains(&"unsafe_blocks".to_string()));
            assert!(patterns.contains(&"blocking_io".to_string()));
            assert!(options.get("check_safety").unwrap());
        } else {
            panic!("Expected Analyze command");
        }

        Ok(())
    }
}
