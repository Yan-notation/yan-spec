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
        self.strip_comments(&text)
    }

    /// Strip '#' line comments and '/* */' block comments while ignoring
    /// both inside single/double-quoted strings (so values like
    /// @color "#ff0080" are not mistaken for a comment start).
    fn strip_comments(&self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len();
        let mut result = String::new();
        let mut i = 0;
        let mut in_quote: Option<char> = None;

        while i < n {
            let ch = chars[i];

            if let Some(q) = in_quote {
                result.push(ch);
                if ch == '\\' && i + 1 < n {
                    result.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                if ch == q {
                    in_quote = None;
                }
                i += 1;
                continue;
            }

            if ch == '"' || ch == '\'' {
                in_quote = Some(ch);
                result.push(ch);
                i += 1;
                continue;
            }

            if ch == '#' {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }

            if ch == '/' && i + 1 < n && chars[i + 1] == '*' {
                i += 2;
                while i + 1 < n && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i = (i + 2).min(n);
                continue;
            }

            result.push(ch);
            i += 1;
        }

        result
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
                        let inner = &content[1..];
                        let value = if self.is_inline_array(inner) {
                            self.parse_array(inner, lines[i].line_num)?
                        } else {
                            YANValue::Object(self.parse_inline_pairs(inner)?)
                        };
                        return Ok((value, i + 1));
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

        if value.starts_with('{') && value.ends_with('}') {
            let inner = &value[1..value.len() - 1];
            return if self.is_inline_array(inner) {
                self.parse_array(inner, line_num)
            } else {
                Ok(YANValue::Object(self.parse_inline_pairs(inner)?))
            };
        }

        if self.is_array(value) {
            return self.parse_array(value, line_num);
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
        let mut brace_depth = 0;
        for ch in text.chars() {
            if ch == '"' { in_quotes = !in_quotes; }
            else if ch == '{' && !in_quotes { brace_depth += 1; }
            else if ch == '}' && !in_quotes { brace_depth -= 1; }
            else if ch == ';' && !in_quotes && brace_depth == 0 { return true; }
        }
        false
    }

    fn has_top_level_colon(&self, text: &str) -> bool {
        let mut in_quotes = false;
        let mut brace_depth = 0;
        for ch in text.chars() {
            if ch == '"' { in_quotes = !in_quotes; }
            else if ch == '{' && !in_quotes { brace_depth += 1; }
            else if ch == '}' && !in_quotes { brace_depth -= 1; }
            else if ch == ':' && !in_quotes && brace_depth == 0 { return true; }
        }
        false
    }

    /// A `{ ... }` block is an array if none of its top-level,
    /// comma/semicolon-separated items has a top-level `key:` colon.
    fn is_inline_array(&self, text: &str) -> bool {
        let items = self.smart_split(text, &[';', ',']);
        !items.iter().any(|item| self.has_top_level_colon(item.trim()))
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
            "bigint" => {
                if raw_value.trim_start_matches('-').is_empty()
                    || !raw_value.trim_start_matches('-').chars().all(|c| c.is_ascii_digit())
                {
                    return Err(YANParseError {
                        message: format!("Invalid @bigint value on line {}: \"{}\"", line_num, raw_value),
                        line: line_num,
                    });
                }
                YANValue::TypeHint {
                    type_name: "bigint".to_string(),
                    value: Box::new(YANValue::String(raw_value.to_string())),
                }
            }
            "email" => {
                let valid = match raw_value.find('@') {
                    Some(at) if at > 0 => {
                        let domain = &raw_value[at + 1..];
                        domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
                    }
                    _ => false,
                };
                if !valid {
                    return Err(YANParseError {
                        message: format!("Invalid @email value on line {}: \"{}\"", line_num, raw_value),
                        line: line_num,
                    });
                }
                YANValue::TypeHint {
                    type_name: "email".to_string(),
                    value: Box::new(YANValue::String(raw_value.to_string())),
                }
            }
            "ipv4" => {
                let parts: Vec<&str> = raw_value.split('.').collect();
                let valid = parts.len() == 4
                    && parts.iter().all(|p| {
                        !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()) && p.parse::<u32>().map(|n| n <= 255).unwrap_or(false)
                    });
                if !valid {
                    return Err(YANParseError {
                        message: format!("Invalid @ipv4 value on line {}: \"{}\"", line_num, raw_value),
                        line: line_num,
                    });
                }
                YANValue::TypeHint {
                    type_name: "ipv4".to_string(),
                    value: Box::new(YANValue::String(raw_value.to_string())),
                }
            }
            "ipv6" => {
                let valid = raw_value.contains(':')
                    && !raw_value.is_empty()
                    && raw_value.chars().all(|c| c.is_ascii_hexdigit() || c == ':');
                if !valid {
                    return Err(YANParseError {
                        message: format!("Invalid @ipv6 value on line {}: \"{}\"", line_num, raw_value),
                        line: line_num,
                    });
                }
                YANValue::TypeHint {
                    type_name: "ipv6".to_string(),
                    value: Box::new(YANValue::String(raw_value.to_string())),
                }
            }
            "color" => {
                let mut cv = raw_value.trim();
                if cv.len() >= 2 {
                    let first = cv.chars().next().unwrap();
                    let last = cv.chars().last().unwrap();
                    if (first == '"' || first == '\'') && first == last {
                        cv = &cv[1..cv.len() - 1];
                    }
                }
                let valid = cv.starts_with('#')
                    && matches!(cv.len() - 1, 3 | 6 | 8)
                    && cv[1..].chars().all(|c| c.is_ascii_hexdigit());
                if !valid {
                    return Err(YANParseError {
                        message: format!(
                            "Invalid @color value on line {}: \"{}\" (note: hex colors must be quoted, e.g. @color \"#ff0080\", since unquoted '#' starts a comment)",
                            line_num, raw_value
                        ),
                        line: line_num,
                    });
                }
                YANValue::TypeHint {
                    type_name: "color".to_string(),
                    value: Box::new(YANValue::String(cv.to_string())),
                }
            }
            "duration" => {
                let mut s = raw_value.strip_prefix('-').unwrap_or(raw_value);
                let mut valid = !s.is_empty();
                while valid && !s.is_empty() {
                    let digit_start = s.len();
                    while s.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                        s = &s[1..];
                    }
                    if s.len() == digit_start { valid = false; break; }
                    if s.starts_with('.') {
                        s = &s[1..];
                        let frac_start = s.len();
                        while s.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                            s = &s[1..];
                        }
                        if s.len() == frac_start { valid = false; break; }
                    }
                    if s.starts_with("ms") { s = &s[2..]; }
                    else if s.starts_with(['d', 'h', 'm', 's']) { s = &s[1..]; }
                    else { valid = false; break; }
                }
                if !valid {
                    return Err(YANParseError {
                        message: format!("Invalid @duration value on line {}: \"{}\"", line_num, raw_value),
                        line: line_num,
                    });
                }
                YANValue::TypeHint {
                    type_name: "duration".to_string(),
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


