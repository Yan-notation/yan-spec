# YAN Specification v1.0

**Yet Another Notation**

A human-readable, human-writable, and machine-readable data interchange format.

---

## 1. Introduction

YAN is a lightweight, text-based, language-independent data interchange format. It is designed to be easy for humans to read and write while remaining trivial for machines to parse and generate.

YAN is programming language independent. It is not derived from any single programming language.

### 1.1 Design Goals

- **Human-readable**: No unnecessary quotes, brackets, or escaping.
- **Human-writable**: Intuitive syntax with comments and flexible formatting.
- **Machine-readable**: Unambiguous grammar with formal specification.
- **Language-independent**: Not tied to any specific programming language.

### 1.2 Conventions Used in This Document

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in RFC 2119.

---

## 2. Grammar

### 2.1 Encoding

A YAN document MUST be encoded in UTF-8. Other encodings are NOT RECOMMENDED and MAY be rejected by parsers.

### 2.2 Whitespace

Whitespace is defined as any of the following Unicode code points:

- `U+0020` SPACE
- `U+0009` CHARACTER TABULATION (tab)
- `U+000A` LINE FEED (LF)
- `U+000D` CARRIAGE RETURN (CR)

A YAN parser MUST normalize `CRLF` (`U+000D U+000A`) and standalone `CR` (`U+000D`) to `LF` (`U+000A`) before parsing.

### 2.3 Indentation

Indentation is used to denote block-level nested objects. An indented block MUST use either spaces or tabs, but MUST NOT mix them within the same document.

The RECOMMENDED indentation is 2 spaces per level.

### 2.4 ABNF Grammar

```abnf
yan-document    = *ws *(comment / pair / ws) *ws

pair            = key ws ":" ws value

key             = 1*key-char
key-char        = ALPHA / DIGIT / "_" / "-"

value           = string
                / number
                / boolean
                / null-value
                / array
                / object-block
                / object-inline
                / type-hint
                / unquoted-string

string          = DQUOTE *char DQUOTE
char            = unescaped / escape
unescaped       = %x20-21 / %x23-5B / %x5D-10FFFF
escape          = "\\" (DQUOTE / "\\" / "b" / "f" / "n" / "r" / "t" / "u" HEXDIG HEXDIG HEXDIG HEXDIG)

unquoted-string = 1*unquoted-char
unquoted-char   = %x21-3A / %x3C-5B / %x5D-7E
                ; printable ASCII excluding : ; { } [ ] @ " \ and whitespace

number          = ["-"] int [frac] [exp]
int             = "0" / ("1"-"9" *DIGIT)
frac            = "." 1*DIGIT
exp             = ("e" / "E") ["-" / "+"] 1*DIGIT

boolean         = "true" / "false" / "yes" / "no" / "on" / "off"

null-value      = "null" / "nil" / "_" / "~"

array           = value *(ws ";" ws value)

object-block    = *(pair ws)
                ; pairs separated by line breaks with increasing indentation

object-inline   = "{" ws *(pair [ws (";" / ",") ws]) ws "}"

type-hint       = "@" type-name ws value
type-name       = 1*(ALPHA / DIGIT / "_" / "-")

comment         = line-comment / block-comment
line-comment    = "#" *VCHAR
block-comment   = "/*" *(*VCHAR / ws) "*/"

ws              = *(SP / HTAB / LF)
```

---

## 3. Data Types

### 3.1 String

A string is a sequence of zero or more Unicode characters.

Strings MAY be quoted with double quotes (`"`). Quoted strings MUST use `"` for escaping double quotes and `\` for escaping backslashes.

Unquoted strings are permitted when the value contains no whitespace, no special characters, and does not conflict with other value types. An unquoted string MUST NOT begin with `@`, `{`, `[`, `"`, `-` (unless it is a negative number), or a digit (unless it is a number).

```yan
name: Budi
greeting: "Hello, World!"
path: "/usr/local/bin"
```

### 3.2 Number

Numbers are represented in base 10. They MAY be integers or floating-point numbers.

```yan
age: 25
pi: 3.14159
negative: -42
scientific: 1.23e-4
```

### 3.3 Boolean

Boolean values represent true/false states. The following literals are recognized:

| Value | Meaning |
|-------|---------|
| `true` | True |
| `false` | False |
| `yes` | True |
| `no` | False |
| `on` | True |
| `off` | False |

```yan
debug: off
ssl: yes
active: true
```

### 3.4 Null

A null value represents the absence of a value. The following literals are recognized:

| Value | Meaning |
|-------|---------|
| `null` | Null |
| `nil` | Null |
| `_` | Null |
| `~` | Null |

```yan
data: null
author: _
optional: ~
```

### 3.5 Array

An array is an ordered list of values separated by semicolons (`;`).

```yan
hobbies: makan; tidur; ngoding
numbers: 1; 2; 3; 5; 8
```

Arrays MAY contain values of different types:

```yan
mixed: hello; 42; true; null
```

### 3.6 Object

An object is an unordered collection of key-value pairs. Objects can be represented in two forms:

#### Block Form (Indentation)

```yan
person:
  name: Budi
  age: 25
  address:
    city: Jakarta
    country: Indonesia
```

#### Inline Form (Braces)

```yan
person: {name: Budi; age: 25}
```

Inline objects MAY use semicolons (`;`) or commas (`,`) as separators. Trailing separators are permitted and MUST be ignored by the parser.

```yan
config: {host: localhost; port: 8080;}
```

### 3.7 Type Hints

Type hints provide explicit type information for values. They are prefixed with `@` followed by a type name and a value.

```yan
created: @datetime 2026-06-27T13:00:00Z
birthdate: @date 2000-05-15
hash: @hex a1b2c3d4
```

#### Core Type Hints

| Hint | Description | Example Output |
|------|-------------|----------------|
| `@int` | Integer | `@int 42` |
| `@float` | Floating-point | `@float 3.14` |
| `@date` | Date (ISO 8601) | `@date 2026-06-27` |
| `@datetime` | Date and time (ISO 8601) | `@datetime 2026-06-27T13:00:00Z` |
| `@hex` | Hexadecimal string | `@hex deadbeef` |
| `@base64` | Base64-encoded string | `@base64 SGVsbG8=` |
| `@uuid` | UUID | `@uuid 550e8400-e29b-41d4-a716-446655440000` |
| `@url` | URL | `@url https://example.com` |
| `@regex` | Regular expression | `@regex [a-z]+` |
| `@bool` | Explicit boolean | `@bool yes` |
| `@bigint` | Arbitrary-precision integer | `@bigint 123456789012345678901234567890` |
| `@email` | Email address | `@email budi@example.com` |
| `@ipv4` | IPv4 address | `@ipv4 192.168.1.1` |
| `@ipv6` | IPv6 address | `@ipv6 2001:db8::1` |
| `@color` | Hex color code | `@color "#ff0080"` |
| `@duration` | Duration (e.g. `1d`, `2h30m`, `500ms`) | `@duration 1h30m` |

Parsers SHOULD support all core type hints. Parsers MAY support additional type hints.

> **Note:** `@color` values MUST be quoted (e.g. `@color "#ff0080"`), since
> an unquoted `#` begins a line comment (see §3.x Comments). Conformant
> parsers MUST be quote-aware when stripping comments, so that a `#`
> inside a quoted string is never treated as the start of a comment.

---

## 4. Comments

### 4.1 Line Comments

Line comments begin with `#` and extend to the end of the line.

```yan
# This is a comment
name: Budi  # This is also a comment
```

### 4.2 Block Comments

Block comments begin with `/*` and end with `*/`. They MAY span multiple lines. Block comments MUST NOT be nested.

```yan
/* This is a
   multi-line comment */
name: Budi
```

### 4.3 Comment Handling

Comments MUST be treated as whitespace by the parser. They MUST NOT appear in the parsed output.

---

## 5. Document Structure

### 5.1 Single Document

A YAN file contains a single document consisting of key-value pairs.

```yan
app:
  name: MyApp
  version: 1.0.0

database:
  host: localhost
  port: 5432
```

### 5.2 Multi-Document (Optional)

Multiple documents MAY be separated by a document marker `---` on its own line.

```yan
---
name: Document One
value: 1
---
name: Document Two
value: 2
```

Multi-document support is OPTIONAL. Parsers that do not support it SHOULD treat `---` as a regular unquoted string.

---

## 6. Error Handling

### 6.1 Parse Errors

A YAN parser MUST produce clear, actionable error messages. Error messages SHOULD include:

- Line number
- Column number (where applicable)
- Description of the error
- Expected vs. actual token

### 6.2 Common Errors

| Error | Description |
|-------|-------------|
| `UNEXPECTED_TOKEN` | A character or token that does not fit the grammar |
| `UNCLOSED_STRING` | A quoted string without a closing `"` |
| `UNCLOSED_BLOCK` | An inline object `{` without a matching `}` |
| `INDENTATION_ERROR` | Mixed tabs and spaces, or inconsistent indentation |
| `DUPLICATE_KEY` | The same key appears twice in the same object |
| `INVALID_TYPE_HINT` | An unknown or malformed type hint |

### 6.3 Duplicate Keys

When a key appears more than once in the same object, the parser SHOULD use the last occurrence. Alternatively, the parser MAY raise a `DUPLICATE_KEY` error.

---

## 7. Compliance Levels

### Level 1: Basic

Parsers MUST support:
- Key-value pairs
- Strings (quoted and unquoted)
- Numbers (integers and floats)
- Booleans (`true`, `false`)
- Null (`null`)
- Arrays with semicolon separators
- Block objects (indentation)
- Inline objects (braces)

### Level 2: Full

Parsers MUST support all of Level 1, plus:
- All boolean aliases (`yes`, `no`, `on`, `off`)
- All null aliases (`nil`, `_`, `~`)
- Comments (`//` and `/* */`)
- All core type hints (`@int`, `@float`, `@date`, `@datetime`, `@hex`, `@base64`, `@uuid`, `@url`, `@regex`, `@bool`)
- Trailing separators in inline objects and arrays

### Level 3: Advanced (Optional)

Parsers MAY support:
- Multi-document files (`---` separator)
- Schema validation
- Streaming parse (YANL)
- Custom type hints

---

## 8. Media Type

The RECOMMENDED media type for YAN documents is:

```
application/yan
```

The RECOMMENDED file extension is:

```
.yan
```

For YAN Lines (one-line documents), the RECOMMENDED media type is:

```
application/yanl
```

The RECOMMENDED file extension is:

```
.yanl
```

---

## 9. Comparison with Other Formats

| Feature | JSON | YAML | TOML | YAN |
|---------|------|------|------|-----|
| Quotes on keys | Required | Optional | Optional | Not required |
| Comments | No | Yes | Yes | Yes |
| Trailing commas | No | Yes | Yes | Yes |
| Type hints | No | No | No | Yes |
| Array syntax | `[a, b]` | `- a` | `[a, b]` | `a; b` |
| Nested objects | `{}` | Indentation | `[table]` | Indentation + `{}` |
| Human readability | Medium | High | High | High |
| Machine readability | High | Medium | High | High |

---

## 10. Examples

### 10.1 Simple Configuration

```yan
// Server configuration
server:
  host: 0.0.0.0
  port: @int 8080
  debug: off

// Database settings
database:
  driver: postgresql
  host: localhost
  port: 5432
  username: admin
  password: "secret123!"
  pool: {min: 5; max: 20;}
```

### 10.2 Complex Document

```yan
/*
 * User profile document
 * Version: 1.0
 */
user:
  id: @uuid 550e8400-e29b-41d4-a716-446655440000
  name: Budi Santoso
  email: budi@example.com
  active: yes
  role: admin

  profile:
    bio: "Software engineer from Jakarta"
    avatar: @url https://cdn.example.com/avatars/budi.png
    joined: @datetime 2020-01-15T08:30:00Z

  preferences:
    theme: dark
    notifications: {email: yes; push: no; sms: _}

  tags: developer; backend; golang; yan
```

### 10.3 YANL (One-Liner)

```yanl
// access.log.yanl
ts: @datetime 2026-06-27T12:00:00Z; method: GET; path: /api/users; status: 200; duration_ms: 45
ts: @datetime 2026-06-27T12:00:01Z; method: POST; path: /api/login; status: 401; error: "Invalid credentials"
```

---

## 11. Security Considerations

YAN parsers SHOULD be careful when evaluating type hints, especially `@regex` and `@url`, to avoid security vulnerabilities such as ReDoS (Regular Expression Denial of Service) or SSRF (Server-Side Request Forgery).

Parsers MUST NOT execute arbitrary code embedded in YAN documents.

---

## 12. References

- RFC 2119: Key words for use in RFCs to Indicate Requirement Levels
- RFC 8259: The JavaScript Object Notation (JSON) Data Interchange Format
- ECMA-404: The JSON Data Interchange Format
- ISO/IEC 21778:2017: Information technology — The JSON data interchange syntax

---

## 13. Authors & Contributors

YAN was designed by [contributors].

For the latest version of this specification, visit:
https://github.com/yan-notation/yan-spec

---

*This specification is released under the CC0 1.0 Universal License.*
