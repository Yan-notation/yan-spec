package yan

import (
	"fmt"
	"regexp"
	"strconv"
	"strings"
)

// YANValue represents any value in YAN
type YANValue interface{}

// YANParseError represents a parse error
type YANParseError struct {
	Message string
	Line    int
}

func (e *YANParseError) Error() string {
	return fmt.Sprintf("YAN parse error at line %d: %s", e.Line, e.Message)
}

// YANParser parses YAN documents
type YANParser struct{}

// NewYANParser creates a new parser
func NewYANParser() *YANParser {
	return &YANParser{}
}

// Parse parses a YAN source string
func (p *YANParser) Parse(source string) (map[string]YANValue, error) {
	cleaned := p.preprocess(source)
	lines := p.splitLines(cleaned)
	result, _, err := p.parseBlock(lines, 0, -1)
	if err != nil {
		return nil, err
	}
	return result, nil
}

// Stringify converts a Go map to YAN string
func (p *YANParser) Stringify(obj map[string]YANValue, indent int, level int) string {
	prefix := strings.Repeat(" ", level*indent)
	var lines []string

	for key, value := range obj {
		switch v := value.(type) {
		case map[string]YANValue:
			sub := p.Stringify(v, indent, level+1)
			if strings.HasPrefix(sub, "{") {
				lines = append(lines, fmt.Sprintf("%s%s: %s", prefix, key, sub))
			} else {
				lines = append(lines, fmt.Sprintf("%s%s:", prefix, key))
				lines = append(lines, sub)
			}
		default:
			lines = append(lines, fmt.Sprintf("%s%s: %s", prefix, key, p.stringifyValue(value)))
		}
	}

	return strings.Join(lines, "\n")
}

func (p *YANParser) stringifyValue(value YANValue) string {
	switch v := value.(type) {
	case nil:
		return "null"
	case bool:
		if v {
			return "true"
		}
		return "false"
	case int:
		return strconv.Itoa(v)
	case int64:
		return strconv.FormatInt(v, 10)
	case float64:
		return strconv.FormatFloat(v, 'f', -1, 64)
	case string:
		if strings.ContainsAny(v, ":;{}[]@\"\n\r# /") || strings.TrimSpace(v) != v {
			return fmt.Sprintf("\"%s\"", strings.ReplaceAll(v, "\"", "\\\""))
		}
		return v
	case []YANValue:
		var items []string
		for _, item := range v {
			items = append(items, p.stringifyValue(item))
		}
		return strings.Join(items, "; ")
	case map[string]YANValue:
		if len(v) == 0 {
			return "{}"
		}
		var pairs []string
		for k, val := range v {
			pairs = append(pairs, fmt.Sprintf("%s: %s", k, p.stringifyValue(val)))
		}
		return fmt.Sprintf("{%s}", strings.Join(pairs, "; "))
	default:
		return fmt.Sprintf("%v", v)
	}
}

func (p *YANParser) preprocess(source string) string {
	text := strings.ReplaceAll(source, "\r\n", "\n")
	text = strings.ReplaceAll(text, "\r", "\n")

	// Remove block comments
	blockCommentRe := regexp.MustCompile(`(?s)/\*.*?\*/`)
	text = blockCommentRe.ReplaceAllString(text, "")

	// Remove line comments
	lines := strings.Split(text, "\n")
	var result []string
	for _, line := range lines {
		if idx := strings.Index(line, "#"); idx != -1 {
			result = append(result, line[:idx])
		} else {
			result = append(result, line)
		}
	}

	return strings.Join(result, "\n")
}

type line struct {
	lineNum int
	indent  int
	content string
}

func (p *YANParser) splitLines(text string) []line {
	var lines []line
	for i, raw := range strings.Split(text, "\n") {
		normalized := strings.ReplaceAll(raw, "\t", "  ")
		trimmed := strings.TrimSpace(normalized)
		if trimmed == "" {
			continue
		}
		indent := len(normalized) - len(strings.TrimLeft(normalized, " "))
		lines = append(lines, line{lineNum: i + 1, indent: indent, content: trimmed})
	}
	return lines
}

func (p *YANParser) parseBlock(lines []line, start int, baseIndent int) (map[string]YANValue, int, error) {
	result := make(map[string]YANValue)
	i := start

	for i < len(lines) {
		ln := lines[i]
		if ln.indent <= baseIndent {
			break
		}

		colonIdx := strings.Index(ln.content, ":")
		if colonIdx == -1 {
			return nil, i, &YANParseError{Message: fmt.Sprintf("Expected ':' in '%s'", ln.content), Line: ln.lineNum}
		}

		key := strings.TrimSpace(ln.content[:colonIdx])
		rawValue := strings.TrimSpace(ln.content[colonIdx+1:])

		if strings.HasPrefix(rawValue, "{") {
			value, nextIdx, err := p.parseInlineObject(lines, i, rawValue)
			if err != nil {
				return nil, i, err
			}
			result[key] = value
			i = nextIdx
			continue
		}

		if i+1 < len(lines) && lines[i+1].indent > ln.indent && rawValue == "" {
			blockValue, nextIdx, err := p.parseBlock(lines, i+1, ln.indent)
			if err != nil {
				return nil, i, err
			}
			result[key] = blockValue
			i = nextIdx
			continue
		}

		value, err := p.parseValue(rawValue, ln.lineNum)
		if err != nil {
			return nil, i, err
		}
		result[key] = value
		i++
	}

	return result, i, nil
}

func (p *YANParser) parseInlineObject(lines []line, start int, rawValue string) (YANValue, int, error) {
	braceCount := 0
	var content strings.Builder
	i := start

	for i < len(lines) {
		text := rawValue
		if i != start {
			text = lines[i].content
		}

		for k, ch := range text {
			if ch == '{' {
				braceCount++
			} else if ch == '}' {
				braceCount--
				if braceCount == 0 {
					content.WriteString(text[:k])
					obj, err := p.parseInlinePairs(content.String()[1:])
					if err != nil {
						return nil, i, err
					}
					return obj, i + 1, nil
				}
			}
		}

		content.WriteString(text)
		content.WriteString(" ")
		i++
	}

	return nil, i, &YANParseError{Message: "Unclosed '{'", Line: lines[start].lineNum}
}

func (p *YANParser) parseInlinePairs(text string) (map[string]YANValue, error) {
	result := make(map[string]YANValue)
	pairs := p.smartSplit(text, []rune{';', ','})

	for _, pair := range pairs {
		trimmed := strings.TrimSpace(pair)
		if trimmed == "" {
			continue
		}
		colonIdx := p.findKeyColon(trimmed)
		if colonIdx == -1 {
			continue
		}
		key := strings.TrimSpace(trimmed[:colonIdx])
		value := strings.TrimSpace(trimmed[colonIdx+1:])
		parsed, err := p.parseValue(value, 0)
		if err != nil {
			return nil, err
		}
		result[key] = parsed
	}

	return result, nil
}

func (p *YANParser) findKeyColon(text string) int {
	inQuotes := false
	for i, ch := range text {
		if ch == '"' && !inQuotes {
			inQuotes = true
		} else if ch == '"' && inQuotes {
			inQuotes = false
		} else if ch == ':' && !inQuotes {
			return i
		}
	}
	return -1
}

func (p *YANParser) smartSplit(text string, delimiters []rune) []string {
	var parts []string
	var current strings.Builder
	inQuotes := false
	braceDepth := 0

	for _, ch := range text {
		if ch == '"' && !inQuotes {
			inQuotes = true
			current.WriteRune(ch)
		} else if ch == '"' && inQuotes {
			inQuotes = false
			current.WriteRune(ch)
		} else if ch == '{' && !inQuotes {
			braceDepth++
			current.WriteRune(ch)
		} else if ch == '}' && !inQuotes {
			braceDepth--
			current.WriteRune(ch)
		} else if containsRune(delimiters, ch) && !inQuotes && braceDepth == 0 {
			parts = append(parts, current.String())
			current.Reset()
		} else {
			current.WriteRune(ch)
		}
	}

	if strings.TrimSpace(current.String()) != "" {
		parts = append(parts, current.String())
	}

	return parts
}

func containsRune(runes []rune, target rune) bool {
	for _, r := range runes {
		if r == target {
			return true
		}
	}
	return false
}

func (p *YANParser) parseValue(raw string, lineNum int) (YANValue, error) {
	value := strings.TrimSpace(raw)
	if value == "" {
		return nil, nil
	}

	if strings.HasPrefix(value, "@") {
		return p.parseTypeHint(value, lineNum)
	}

	if p.isArray(value) {
		return p.parseArray(value, lineNum)
	}

	if strings.HasPrefix(value, "{") {
		return p.parseInlinePairs(value)
	}

	if strings.HasPrefix(value, "\"") {
		return p.parseString(value, lineNum)
	}

	lower := strings.ToLower(value)
	if lower == "true" || lower == "yes" || lower == "on" {
		return true, nil
	}
	if lower == "false" || lower == "no" || lower == "off" {
		return false, nil
	}
	if lower == "null" || lower == "nil" || lower == "_" || lower == "~" {
		return nil, nil
	}

	if n, err := strconv.ParseInt(value, 10, 64); err == nil {
		return n, nil
	}
	if n, err := strconv.ParseFloat(value, 64); err == nil {
		return n, nil
	}

	return value, nil
}

func (p *YANParser) isArray(text string) bool {
	inQuotes := false
	for _, ch := range text {
		if ch == '"' {
			inQuotes = !inQuotes
		} else if ch == ';' && !inQuotes {
			return true
		}
	}
	return false
}

func (p *YANParser) parseArray(text string, lineNum int) (YANValue, error) {
	items := p.smartSplit(text, []rune{';'})
	var result []YANValue
	for _, item := range items {
		trimmed := strings.TrimSpace(item)
		if trimmed == "" {
			continue
		}
		value, err := p.parseValue(trimmed, lineNum)
		if err != nil {
			return nil, err
		}
		result = append(result, value)
	}
	return result, nil
}

func (p *YANParser) parseString(text string, lineNum int) (YANValue, error) {
	if !strings.HasPrefix(text, "\"") {
		return text, nil
	}
	if !strings.HasSuffix(text, "\"") {
		return nil, &YANParseError{Message: fmt.Sprintf("Unclosed string: %s", text), Line: lineNum}
	}
	s := text[1 : len(text)-1]
	s = strings.ReplaceAll(s, "\\\"", "\"")
	return s, nil
}

func (p *YANParser) parseTypeHint(text string, lineNum int) (YANValue, error) {
	spaceIdx := strings.Index(text, " ")
	typeName := text[1:]
	rawValue := ""
	if spaceIdx != -1 {
		typeName = text[1:spaceIdx]
		rawValue = text[spaceIdx+1:]
	}

	switch typeName {
	case "int":
		n, err := strconv.ParseInt(rawValue, 10, 64)
		if err != nil {
			return nil, &YANParseError{Message: fmt.Sprintf("Invalid int: %s", rawValue), Line: lineNum}
		}
		return n, nil
	case "float":
		n, err := strconv.ParseFloat(rawValue, 64)
		if err != nil {
			return nil, &YANParseError{Message: fmt.Sprintf("Invalid float: %s", rawValue), Line: lineNum}
		}
		return n, nil
	case "bool":
		return strings.Contains("true yes on 1", strings.ToLower(rawValue)), nil
	case "date", "datetime", "hex", "base64", "uuid", "url", "regex":
		return map[string]YANValue{
			"__type":  typeName,
			"__value": rawValue,
		}, nil
	default:
		value, err := p.parseValue(rawValue, lineNum)
		if err != nil {
			return nil, err
		}
		return map[string]YANValue{
			"__type":  typeName,
			"__value": value,
		}, nil
	}
}
