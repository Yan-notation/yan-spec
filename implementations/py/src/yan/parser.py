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
        # Remove block comments
        text = re.sub(r'/\*.*?\*/', '', text, flags=re.DOTALL)
        # Remove line comments
        text = re.sub(r'#.*$', '', text, flags=re.MULTILINE)
        return text

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

    def _parse_inline_object(self, lines: List[dict], start: int, raw_value: str) -> Tuple[dict, int]:
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
                        obj = self._parse_inline_pairs(content[1:])  # skip first {
                        return obj, i + 1

            content += text + " "
            i += 1

        raise YANParseError(f"Unclosed '{{' starting near line {lines[start]['line']}")

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

        # Array
        if self._is_array(value):
            return self._parse_array(value, line_num)

        # Inline object
        if value.startswith("{"):
            return self._parse_inline_pairs(value)

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
        for ch in text:
            if ch == '"':
                in_quotes = not in_quotes
            elif ch == ";" and not in_quotes:
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
        }

        if type_name in handlers:
            return handlers[type_name](raw_value)

        return {"__type": type_name, "__value": self._parse_value(raw_value, line_num)}
