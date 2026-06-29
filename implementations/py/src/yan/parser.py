"""YAN Parser v1.0 for Python."""

import re
from typing import Any, Dict, List, Tuple, Union


class YANParseError(Exception):
    """Raised when a YAN document cannot be parsed."""
    pass


class YANParser:
    """Parse YAN source strings into Python objects and back."""

    def __init__(self, strict: bool = False):
        self.strict = strict

    # ==================== PUBLIC API ====================

    def parse(self, source: str) -> dict:
        """Parse a YAN source string into a Python dict."""
        cleaned = self._preprocess(source)
        lines = self._split_lines(cleaned)
        result, _ = self._parse_block(lines, 0, -1)
        return result

    def stringify(self, obj: Any, indent: int = 2, level: int = 0, inline: bool = False) -> str:
        """Convert a Python object to a YAN string."""
        prefix = " " * (level * indent)

        if obj is None:
            return "null"

        if isinstance(obj, bool):
            return "true" if obj else "false"

        if isinstance(obj, int):
            return str(obj)

        if isinstance(obj, float):
            s = str(obj)
            return s if "." in s or "e" in s.lower() else s + ".0"

        if isinstance(obj, str):
            if re.search(r'[:;{}\[\]@"\n\r#\/\s]', obj) or obj.strip() != obj:
                return '"' + obj.replace('"', '\\"') + '"'
            return obj

        if isinstance(obj, (list, tuple)):
            return "; ".join(self.stringify(v, indent, 0, True) for v in obj)

        if isinstance(obj, dict):
            if not obj:
                return "{}"

            entries = list(obj.items())
            if inline or len(entries) <= 2:
                pairs = "; ".join(
                    f"{k}: {self.stringify(v, indent, 0, True)}"
                    for k, v in entries
                )
                return "{" + pairs + "}"

            lines = []
            for key, value in obj.items():
                if isinstance(value, dict) and value is not None:
                    sub = self.stringify(value, indent, level + 1)
                    if sub.startswith("{"):
                        lines.append(f"{prefix}{key}: {sub}")
                    else:
                        lines.append(f"{prefix}{key}:")
                        lines.append(sub)
                else:
                    lines.append(f"{prefix}{key}: {self.stringify(value, indent, 0, True)}")
            return "\n".join(lines)

        return str(obj)

    # ==================== INTERNAL METHODS ====================

    def _preprocess(self, source: str) -> str:
        text = source.replace("\r\n", "\n").replace("\r", "\n")
        return self._strip_comments(text)

    def _strip_comments(self, text: str) -> str:
        """Strip '#' line comments and '/* */' block comments while
        ignoring both inside single/double-quoted strings (so values
        like @color "#ff0080" are not mistaken for a comment start)."""
        result = []
        i = 0
        n = len(text)
        in_quote = None

        while i < n:
            ch = text[i]

            if in_quote:
                result.append(ch)
                if ch == '\\' and i + 1 < n:
                    result.append(text[i + 1])
                    i += 2
                    continue
                if ch == in_quote:
                    in_quote = None
                i += 1
                continue

            if ch in ('"', "'"):
                in_quote = ch
                result.append(ch)
                i += 1
                continue

            if ch == '#':
                eol = text.find('\n', i)
                i = n if eol == -1 else eol
                continue

            if ch == '/' and i + 1 < n and text[i + 1] == '*':
                close = text.find('*/', i + 2)
                i = n if close == -1 else close + 2
                continue

            result.append(ch)
            i += 1

        return ''.join(result)

    def _split_lines(self, text: str) -> List[dict]:
        lines = []
        for i, raw in enumerate(text.split("\n"), start=1):
            normalized = raw.replace("\t", "  ")
            stripped = normalized.rstrip()
            if not stripped:
                continue
            indent = len(normalized) - len(normalized.lstrip())
            content = stripped.lstrip()
            lines.append({"line": i, "indent": indent, "content": content})
        return lines

    def _parse_block(self, lines: List[dict], start: int, base_indent: int) -> Tuple[dict, int]:
        result = {}
        i = start

        while i < len(lines):
            line = lines[i]

            if line["indent"] <= base_indent:
                break

            colon_idx = line["content"].find(":")
            if colon_idx == -1:
                raise YANParseError(
                    f"Expected ':' on line {line['line']}: '{line['content']}'"
                )

            key = line["content"][:colon_idx].strip()
            raw_value = line["content"][colon_idx + 1:].strip()

            # Inline object
            if raw_value.startswith("{"):
                value, i = self._parse_inline_object(lines, i, raw_value)
                result[key] = value
                continue

            # Check if next line is indented more (block value)
            next_line = lines[i + 1] if i + 1 < len(lines) else None
            if next_line and next_line["indent"] > line["indent"] and not raw_value:
                block_value, i = self._parse_block(lines, i + 1, line["indent"])
                result[key] = block_value
                continue

            result[key] = self._parse_value(raw_value, line["line"])
            i += 1

        return result, i

    def _parse_inline_object(self, lines: List[dict], start: int, raw_value: str) -> Tuple[Any, int]:
        brace_count = 0
        content = ""
        i = start

        while i < len(lines):
            line = lines[i]
            text = raw_value if i == start else line["content"]

            for k, ch in enumerate(text):
                if ch == "{":
                    brace_count += 1
                elif ch == "}":
                    brace_count -= 1
                    if brace_count == 0:
                        content += text[:k]
                        inner = content[1:]  # skip first {
                        if self._is_inline_array(inner):
                            value = self._parse_array(inner, lines[i]["line"])
                        else:
                            value = self._parse_inline_pairs(inner)
                        return value, i + 1

            content += text + " "
            i += 1

        raise YANParseError(f"Unclosed '{{' starting near line {lines[start]['line']}")

    def _is_inline_array(self, text: str) -> bool:
        """A `{ ... }` block is an array if none of its top-level,
        comma/semicolon-separated items has a top-level `key:` colon."""
        items = self._smart_split(text, re.compile(r'[,;]'))
        for item in items:
            trimmed = item.strip()
            if not trimmed:
                continue
            if self._has_top_level_colon(trimmed):
                return False
        return True

    def _has_top_level_colon(self, text: str) -> bool:
        in_quotes = False
        brace_depth = 0
        for ch in text:
            if ch == '"' and not in_quotes:
                in_quotes = True
            elif ch == '"' and in_quotes:
                in_quotes = False
            elif ch == "{" and not in_quotes:
                brace_depth += 1
            elif ch == "}" and not in_quotes:
                brace_depth -= 1
            elif ch == ":" and not in_quotes and brace_depth == 0:
                return True
        return False

    def _parse_inline_pairs(self, text: str) -> dict:
        result = {}
        pairs = self._smart_split(text, re.compile(r'[,;]'))

        for pair in pairs:
            trimmed = pair.strip()
            if not trimmed:
                continue
            # Find first colon that's not inside a URL or other value
            colon_idx = self._find_key_colon(trimmed)
            if colon_idx == -1:
                continue
            key = trimmed[:colon_idx].strip()
            value = trimmed[colon_idx + 1:].strip()
            result[key] = self._parse_value(value, 0)

        return result

    def _find_key_colon(self, text: str) -> int:
        """Find the colon that separates key from value."""
        in_quotes = False
        for i, ch in enumerate(text):
            if ch == '"':
                in_quotes = not in_quotes
            elif ch == ':' and not in_quotes:
                return i
        return -1

    def _smart_split(self, text: str, delimiter: re.Pattern) -> List[str]:
        parts = []
        current = ""
        in_quotes = False
        brace_depth = 0

        for ch in text:
            if ch == '"' and not in_quotes:
                in_quotes = True
                current += ch
            elif ch == '"' and in_quotes:
                in_quotes = False
                current += ch
            elif ch == "{" and not in_quotes:
                brace_depth += 1
                current += ch
            elif ch == "}" and not in_quotes:
                brace_depth -= 1
                current += ch
            elif delimiter.match(ch) and not in_quotes and brace_depth == 0:
                parts.append(current)
                current = ""
            else:
                current += ch

        if current.strip():
            parts.append(current)
        return parts

    def _parse_value(self, raw: str, line_num: int) -> Any:
        value = raw.strip()
        if not value:
            return None

        # Type hint
        if value.startswith("@"):
            return self._parse_type_hint(value, line_num)

        # Inline object/array { ... }
        if value.startswith("{") and value.endswith("}"):
            inner = value[1:-1]
            if self._is_inline_array(inner):
                return self._parse_array(inner, line_num)
            return self._parse_inline_pairs(inner)

        # Bare array (contains top-level ; not in quotes/braces)
        if self._is_array(value):
            return self._parse_array(value, line_num)

        # Quoted string
        if value.startswith('"'):
            return self._parse_string(value, line_num)

        # Boolean
        lower = value.lower()
        if lower in ("true", "yes", "on"):
            return True
        if lower in ("false", "no", "off"):
            return False

        # Null
        if lower in ("null", "nil", "_", "~"):
            return None

        # Number
        if re.match(r'^-?\d+(\.\d+)?([eE][+-]?\d+)?$', value):
            return int(value) if "." not in value and "e" not in value.lower() else float(value)

        # Unquoted string
        return value

    def _is_array(self, text: str) -> bool:
        in_quotes = False
        brace_depth = 0
        for ch in text:
            if ch == '"':
                in_quotes = not in_quotes
            elif ch == "{" and not in_quotes:
                brace_depth += 1
            elif ch == "}" and not in_quotes:
                brace_depth -= 1
            elif ch == ";" and not in_quotes and brace_depth == 0:
                return True
        return False

    def _parse_array(self, text: str, line_num: int) -> List[Any]:
        items = self._smart_split(text, re.compile(r';'))
        return [self._parse_value(item.strip(), line_num) for item in items if item.strip()]

    def _parse_string(self, text: str, line_num: int) -> str:
        if not text.startswith('"'):
            return text
        if not text.endswith('"'):
            raise YANParseError(f"Unclosed string on line {line_num}: {text}")
        return text[1:-1].replace('\\"', '"')

    def _parse_type_hint(self, text: str, line_num: int) -> Any:
        space_idx = text.find(" ")
        type_name = text[1:space_idx] if space_idx != -1 else text[1:]
        raw_value = text[space_idx + 1:] if space_idx != -1 else ""

        handlers = {
            "int": lambda v: int(v),
            "float": lambda v: float(v),
            "date": lambda v: {"__type": "date", "__value": v},
            "datetime": lambda v: {"__type": "datetime", "__value": v},
            "hex": lambda v: {"__type": "hex", "__value": v},
            "base64": lambda v: {"__type": "base64", "__value": v},
            "uuid": lambda v: {"__type": "uuid", "__value": v},
            "url": lambda v: {"__type": "url", "__value": v},
            "regex": lambda v: {"__type": "regex", "__value": v},
            "bool": lambda v: v.lower() in ("true", "yes", "on", "1"),
            "bigint": self._handle_bigint,
            "email": self._handle_email,
            "ipv4": self._handle_ipv4,
            "ipv6": self._handle_ipv6,
            "color": lambda v: self._handle_color(v, line_num),
            "duration": lambda v: self._handle_duration(v, line_num),
        }

        if type_name in handlers:
            try:
                return handlers[type_name](raw_value)
            except YANParseError:
                raise
            except (ValueError, TypeError) as e:
                raise YANParseError(f"Invalid @{type_name} value on line {line_num}: \"{raw_value}\"") from e

        return {"__type": type_name, "__value": self._parse_value(raw_value, line_num)}

    def _handle_bigint(self, v: str) -> dict:
        if not re.match(r'^-?\d+$', v):
            raise ValueError(f"not an integer: {v}")
        return {"__type": "bigint", "__value": str(int(v))}

    def _handle_email(self, v: str) -> dict:
        if not re.match(r'^[^\s@]+@[^\s@]+\.[^\s@]+$', v):
            raise ValueError(f"not a valid email: {v}")
        return {"__type": "email", "__value": v}

    def _handle_ipv4(self, v: str) -> dict:
        octets = v.split(".")
        if len(octets) != 4 or not all(o.isdigit() and 0 <= int(o) <= 255 for o in octets):
            raise ValueError(f"not a valid IPv4 address: {v}")
        return {"__type": "ipv4", "__value": v}

    def _handle_ipv6(self, v: str) -> dict:
        if ":" not in v or not re.match(r'^[0-9a-fA-F:]+$', v):
            raise ValueError(f"not a valid IPv6 address: {v}")
        return {"__type": "ipv6", "__value": v}

    def _handle_color(self, v: str, line_num: int) -> dict:
        color_value = v.strip()
        if (color_value.startswith('"') and color_value.endswith('"')) or \
           (color_value.startswith("'") and color_value.endswith("'")):
            color_value = color_value[1:-1]
        if not re.match(r'^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$', color_value):
            raise YANParseError(
                f"Invalid @color value on line {line_num}: \"{v}\" "
                f"(note: hex colors must be quoted, e.g. @color \"#ff0080\", "
                f"since unquoted '#' starts a comment)"
            )
        return {"__type": "color", "__value": color_value}

    def _handle_duration(self, v: str, line_num: int) -> dict:
        if not re.match(r'^-?(\d+(\.\d+)?(d|h|m|s|ms))+$', v):
            raise YANParseError(f"Invalid @duration value on line {line_num}: \"{v}\"")
        return {"__type": "duration", "__value": v}
