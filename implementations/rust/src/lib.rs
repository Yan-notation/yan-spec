use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum YANValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<YANValue>),
    Object(HashMap<String, YANValue>),
    TypeHint { type_name: String, value: Box<YANValue> },
}

#[derive(Debug)]
pub struct YANParseError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for YANParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "YAN parse error at line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for YANParseError {}

pub struct YANParser;

impl YANParser {
    pub fn new() -> Self {
        YANParser
    }

    pub fn parse(&self, source: &str) -> Result<HashMap<String, YANValue>, YANParseError> {
        let cleaned = self.preprocess(source);
        let lines = self.split_lines(&cleaned);
        let (result, _) = self.parse_block(&lines, 0, -1)?;
        Ok(result)
    }

    fn preprocess(&self, source: &str) -> String {
        let text = source.replace("\r\n", "\n").replace('\r', "\n");
        let mut result = String::new();
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '/' && chars.peek() == Some(&'*') {
                chars.next();
                loop {
                    match chars.next() {
                        Some('*') if chars.peek() == Some(&'/') => { chars.next(); break; }
                        None => break,
                        _ => {}
                    }
                }
            } else {
                result.push(ch);
            }
        }
        let mut final_result = String::new();
        for line in result.lines() {
            if let Some(idx) = line.find('#') {
                final_result.push_str(&line[..idx]);
            } else {
                final_result.push_str(line);
            }
            final_result.push('\n');
        }
        final_result
    }

    fn split_lines(&self, text: &str) -> Vec<Line> {
        text.lines()
            .enumerate()
            .map(|(i, line)| {
                let normalized = line.replace('\t', "  ");
                let trimmed = normalized.trim();
                let indent = normalized.len() - normalized.trim_start().len();
                Line {
                    line_num: i + 1,
                    indent,
                    content: trimmed.to_string(),
                }
            })
            .filter(|l| !l.content.is_empty())
            .collect()
    }

    fn parse_block(&self, lines: &[Line], start: usize, base_indent: isize) -> Result<(HashMap<String, YANValue>, usize), YANParseError> {
        let mut result = HashMap::new();
        let mut i = start;

        while i < lines.len() {
            let line = &lines[i];
            if line.indent as isize <= base_indent {
                break;
            }

            let colon_idx = line.content.find(':')
                .ok_or_else(|| YANParseError {
                    message: format!("Expected ':' in '{}'", line.content),
                    line: line.line_num,
                })?;

            let key = line.content[..colon_idx].trim().to_string();
            let raw_value = line.content[colon_idx + 1..].trim().to_string();

            if raw_value.starts_with('{') {
                let (value, next_idx) = self.parse_inline_object(lines, i, &raw_value)?;
                result.insert(key, value);
                i = next_idx;
                continue;
            }

            let next_line = lines.get(i + 1);
            if let Some(next) = next_line {
                if next.indent > line.indent && raw_value.is_empty() {
                    let (block_value, next_idx) = self.parse_block(lines, i + 1, line.indent as isize)?;
                    result.insert(key, YANValue::Object(block_value));
                    i = next_idx;
                    continue;
                }
            }

            result.insert(key, self.parse_value(&raw_value, line.line_num)?);
            i += 1;
        }

        Ok((result, i))
    }

    fn parse_inline_object(&self, lines: &[Line], start: usize, raw_value: &str) -> Result<(YANValue, usize), YANParseError> {
        let mut brace_count = 0;
        let mut content = String::new();
        let mut i = start;

        while i < lines.len() {
            let text = if i == start { raw_value } else { &lines[i].content };
            for (k, ch) in text.chars().enumerate() {
                if ch == '{' {
                    brace_count += 1;
                } else if ch == '}' {
                    brace_count -= 1;
                    if brace_count == 0 {
                        content.push_str(&text[..k]);
                        let obj = self.parse_inline_pairs(&content[1..])?;
                        return Ok((YANValue::Object(obj), i + 1));
                    }
                }
            }
            content.push_str(text);
            content.push(' ');
            i += 1;
        }

        Err(YANParseError {
            message: "Unclosed '{'".to_string(),
            line: lines[start].line_num,
        })
    }

    fn parse_inline_pairs(&self, text: &str) -> Result<HashMap<String, YANValue>, YANParseError> {
        let mut result = HashMap::new();
        let pairs = self.smart_split(text, &[';', ',']);

        for pair in pairs {
            let trimmed = pair.trim();
            if trimmed.is_empty() { continue; }
            let colon_idx = self.find_key_colon(trimmed)
                .ok_or_else(|| YANParseError {
                    message: format!("Expected ':' in '{}'", trimmed),
                    line: 0,
                })?;
            let key = trimmed[..colon_idx].trim().to_string();
            let value = trimmed[colon_idx + 1..].trim();
            result.insert(key, self.parse_value(value, 0)?);
        }

        Ok(result)
    }

    fn find_key_colon(&self, text: &str) -> Option<usize> {
        let mut in_quotes = false;
        for (i, ch) in text.chars().enumerate() {
            if ch == '"' && !in_quotes { in_quotes = true; }
            else if ch == '"' && in_quotes { in_quotes = false; }
            else if ch == ':' && !in_quotes { return Some(i); }
        }
        None
    }

    fn smart_split(&self, text: &str, delimiters: &[char]) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut brace_depth = 0;

        for ch in text.chars() {
            if ch == '"' && !in_quotes { in_quotes = true; current.push(ch); }
            else if ch == '"' && in_quotes { in_quotes = false; current.push(ch); }
            else if ch == '{' && !in_quotes { brace_depth += 1; current.push(ch); }
            else if ch == '}' && !in_quotes { brace_depth -= 1; current.push(ch); }
            else if delimiters.contains(&ch) && !in_quotes && brace_depth == 0 {
                parts.push(current.clone());
                current.clear();
            } else {
                current.push(ch);
            }
        }
        if !current.trim().is_empty() {
            parts.push(current);
        }
        parts
    }

    fn parse_value(&self, raw: &str, line_num: usize) -> Result<YANValue, YANParseError> {
        let value = raw.trim();
        if value.is_empty() { return Ok(YANValue::Null); }

        if value.starts_with('@') {
            return self.parse_type_hint(value, line_num);
        }

        if self.is_array(value) {
            return self.parse_array(value, line_num);
        }

        if value.starts_with('{') {
            let obj = self.parse_inline_pairs(value)?;
            return Ok(YANValue::Object(obj));
        }

        if value.starts_with('"') {
            return self.parse_string(value, line_num);
        }

        let lower = value.to_lowercase();
        if ["true", "yes", "on"].contains(&lower.as_str()) { return Ok(YANValue::Bool(true)); }
        if ["false", "no", "off"].contains(&lower.as_str()) { return Ok(YANValue::Bool(false)); }
        if ["null", "nil", "_", "~"].contains(&lower.as_str()) { return Ok(YANValue::Null); }

        if let Ok(n) = value.parse::<i64>() {
            return Ok(YANValue::Int(n));
        }
        if let Ok(n) = value.parse::<f64>() {
            return Ok(YANValue::Float(n));
        }

        Ok(YANValue::String(value.to_string()))
    }

    fn is_array(&self, text: &str) -> bool {
        let mut in_quotes = false;
        for ch in text.chars() {
            if ch == '"' { in_quotes = !in_quotes; }
            else if ch == ';' && !in_quotes { return true; }
        }
        false
    }

    fn parse_array(&self, text: &str, line_num: usize) -> Result<YANValue, YANParseError> {
        let items = self.smart_split(text, &[';']);
        let values: Result<Vec<_>, _> = items.into_iter()
            .map(|item| self.parse_value(item.trim(), line_num))
            .collect();
        Ok(YANValue::Array(values?))
    }

    fn parse_string(&self, text: &str, line_num: usize) -> Result<YANValue, YANParseError> {
        if !text.starts_with('"') { return Ok(YANValue::String(text.to_string())); }
        if !text.ends_with('"') {
            return Err(YANParseError {
                message: format!("Unclosed string: {}", text),
                line: line_num,
            });
        }
        let s = text[1..text.len()-1].replace("\\\"", "\"");
        Ok(YANValue::String(s))
    }

    fn parse_type_hint(&self, text: &str, line_num: usize) -> Result<YANValue, YANParseError> {
        let space_idx = text.find(' ').unwrap_or(text.len());
        let type_name = &text[1..space_idx];
        let raw_value = if space_idx < text.len() { &text[space_idx + 1..] } else { "" };

        let value = match type_name {
            "int" => YANValue::Int(raw_value.parse().map_err(|_| YANParseError {
                message: format!("Invalid int: {}", raw_value),
                line: line_num,
            })?),
            "float" => YANValue::Float(raw_value.parse().map_err(|_| YANParseError {
                message: format!("Invalid float: {}", raw_value),
                line: line_num,
            })?),
            "bool" => YANValue::Bool(["true", "yes", "on", "1"].contains(&raw_value.to_lowercase().as_str())),
            "date" | "datetime" | "hex" | "base64" | "uuid" | "url" | "regex" => {
                YANValue::TypeHint {
                    type_name: type_name.to_string(),
                    value: Box::new(YANValue::String(raw_value.to_string())),
                }
            }
            _ => YANValue::TypeHint {
                type_name: type_name.to_string(),
                value: Box::new(self.parse_value(raw_value, line_num)?),
            }
        };
        Ok(value)
    }
}

#[derive(Debug)]
struct Line {
    line_num: usize,
    indent: usize,
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_string() {
        let parser = YANParser::new();
        let result = parser.parse("name: Budi").unwrap();
        assert_eq!(result.get("name"), Some(&YANValue::String("Budi".to_string())));
    }

    #[test]
    fn test_primitive_number() {
        let parser = YANParser::new();
        let result = parser.parse("age: 25").unwrap();
        assert_eq!(result.get("age"), Some(&YANValue::Int(25)));
    }

    #[test]
    fn test_boolean() {
        let parser = YANParser::new();
        let result = parser.parse("active: yes").unwrap();
        assert_eq!(result.get("active"), Some(&YANValue::Bool(true)));
    }

    #[test]
    fn test_null() {
        let parser = YANParser::new();
        let result = parser.parse("data: null").unwrap();
        assert_eq!(result.get("data"), Some(&YANValue::Null));
    }

    #[test]
    fn test_array() {
        let parser = YANParser::new();
        let result = parser.parse("tags: a; b; c").unwrap();
        if let Some(YANValue::Array(arr)) = result.get("tags") {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_inline_object() {
        let parser = YANParser::new();
        let result = parser.parse("cfg: {host: localhost; port: 80}").unwrap();
        if let Some(YANValue::Object(obj)) = result.get("cfg") {
            assert_eq!(obj.get("host"), Some(&YANValue::String("localhost".to_string())));
            assert_eq!(obj.get("port"), Some(&YANValue::Int(80)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_block_object() {
        let parser = YANParser::new();
        let result = parser.parse("person:\n  name: Budi\n  age: 25").unwrap();
        if let Some(YANValue::Object(obj)) = result.get("person") {
            assert_eq!(obj.get("name"), Some(&YANValue::String("Budi".to_string())));
            assert_eq!(obj.get("age"), Some(&YANValue::Int(25)));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_comment() {
        let parser = YANParser::new();
        let result = parser.parse("# comment\nname: Budi").unwrap();
        assert_eq!(result.get("name"), Some(&YANValue::String("Budi".to_string())));
    }

    #[test]
    fn test_type_hint() {
        let parser = YANParser::new();
        let result = parser.parse("n: @int 42").unwrap();
        assert_eq!(result.get("n"), Some(&YANValue::Int(42)));
    }
}
