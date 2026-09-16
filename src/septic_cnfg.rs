use anyhow::{Result, bail};
use indexmap::IndexMap;

#[derive(Debug, PartialEq)]
pub(crate) struct Object {
    pub object_type: String,
    pub name: String,
    pub attributes: IndexMap<String, String>,
}

pub(crate) fn parse(input: &str) -> Result<Vec<Object>> {
    let input = strip_comments(input)?;
    let mut objects = Vec::new();
    let mut current: Option<Object> = None;
    let mut current_attribute: Option<String> = None;

    for (line_number, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((object_type, name)) = line.split_once(':')
            && !object_type.contains(char::is_whitespace)
            && !object_type.contains('=')
        {
            if let Some(object) = current.take() {
                objects.push(object);
            }
            current = Some(Object {
                object_type: object_type.trim().to_string(),
                name: name.trim().to_string(),
                attributes: IndexMap::new(),
            });
            current_attribute = None;
            continue;
        }

        if let Some((name, value)) = line.split_once('=') {
            let Some(object) = current.as_mut() else {
                bail!("attribute before first object at line {}", line_number + 1);
            };
            let name = name.trim().to_string();
            object
                .attributes
                .insert(name.clone(), logical_value(value.trim()));
            current_attribute = Some(name);
            continue;
        }

        if let (Some(object), Some(attribute)) = (current.as_mut(), &current_attribute)
            && let Some(value) = object.attributes.get_mut(attribute)
        {
            value.push(' ');
            value.push_str(line);
            continue;
        }

        bail!(
            "unexpected CNFG content at line {}: {line}",
            line_number + 1
        );
    }

    if let Some(object) = current {
        objects.push(object);
    }
    Ok(objects)
}

fn logical_value(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_string()
}

fn strip_comments(input: &str) -> Result<String> {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut quoted = false;

    while let Some(character) = chars.next() {
        if character == '"' {
            quoted = !quoted;
            result.push(character);
        } else if !quoted && character == '/' && chars.peek() == Some(&'/') {
            chars.next();
            for character in chars.by_ref() {
                if character == '\n' {
                    result.push('\n');
                    break;
                }
            }
        } else if !quoted && character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut closed = false;
            while let Some(character) = chars.next() {
                if character == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    closed = true;
                    break;
                }
                if character == '\n' {
                    result.push('\n');
                }
            }
            if !closed {
                bail!("unterminated block comment");
            }
        } else if !quoted && character == '{' && matches!(chars.peek(), Some('#' | '%')) {
            let marker = chars.next().unwrap();
            let mut closed = false;
            while let Some(character) = chars.next() {
                if character == marker && chars.peek() == Some(&'}') {
                    chars.next();
                    closed = true;
                    break;
                }
                if character == '\n' {
                    result.push('\n');
                }
            }
            if !closed {
                bail!("unterminated Jinja block");
            }
        } else {
            result.push(character);
        }
    }

    if quoted {
        bail!("unterminated quoted string");
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_objects_and_logical_values() {
        let input = r#"
  Cvr: D01Qg
      Text1= "D01: Gas rate"
      Meas= 230000
      Blocking= 8 1 2 4

  Mvr: D01Zpc
      Meas= 35
"#;

        let objects = parse(input).unwrap();

        assert_eq!(objects.len(), 2);
        assert_eq!(objects[0].object_type, "Cvr");
        assert_eq!(objects[0].name, "D01Qg");
        assert_eq!(objects[0].attributes["Text1"], "D01: Gas rate");
        assert_eq!(objects[0].attributes["Meas"], "230000");
        assert_eq!(objects[0].attributes["Blocking"], "8 1 2 4");
    }

    #[test]
    fn ignores_comments_and_keeps_comment_markers_in_strings() {
        let input = r#"
// heading
System: test /* inline */
    Text1= "http://example.test/*"
    Nsecs= 10 // seconds
"#;

        let objects = parse(input).unwrap();

        assert_eq!(objects[0].attributes["Text1"], "http://example.test/*");
        assert_eq!(objects[0].attributes["Nsecs"], "10");
    }

    #[test]
    fn duplicate_attributes_use_last_value() {
        let objects = parse("SopcMvr: D01Zpc\nNotValidTag= first\nNotValidTag= second").unwrap();

        assert_eq!(objects[0].attributes["NotValidTag"], "second");
    }

    #[test]
    fn joins_multiline_attribute_values() {
        let objects = parse("CvrList: list\nCvrs= 3\n\"D01Qg\" \"D02Qg\" \"D03Qg\"").unwrap();

        assert_eq!(
            objects[0].attributes["Cvrs"],
            "3 \"D01Qg\" \"D02Qg\" \"D03Qg\""
        );
    }

    #[test]
    fn ignores_jinja_blocks_but_preserves_jinja_identifiers() {
        let objects = parse("Evr: Test{{Something}} {# comment #}\nMeas= 1 {% ignored %}").unwrap();

        assert_eq!(objects[0].name, "Test{{Something}}");
        assert_eq!(objects[0].attributes["Meas"], "1");
    }

    #[test]
    fn parses_repository_example() {
        let objects = parse(include_str!("../docs/basic example/example.cnfg")).unwrap();
        let cvr = objects
            .iter()
            .find(|object| object.object_type == "Cvr" && object.name == "D01Qg")
            .unwrap();

        assert_eq!(cvr.attributes["Meas"], "230000");
    }
}
