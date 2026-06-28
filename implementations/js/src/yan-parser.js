/**
 * YAN Parser v1.0
 * Yet Another Notation
 * 
 * Human readable, writable, and machine readable.
 */

class YANParser {
  constructor(options = {}) {
    this.strict = options.strict ?? false;
    this.preserveComments = options.preserveComments ?? false;
  }

  /**
   * Parse YAN source string into JavaScript object
   */
  parse(source) {
    // Step 1: Preprocess - normalize line endings and remove comments
    const cleaned = this._preprocess(source);

    // Step 2: Split into lines and compute indentation
    const lines = this._splitLines(cleaned);

    // Step 3: Parse document
    const { result } = this._parseBlock(lines, 0, -1);
    return result;
  }

  /**
   * Convert JavaScript object to YAN string
   */
  stringify(obj, options = {}) {
    const indent = options.indent ?? 2;
    const level = options.level ?? 0;
    const prefix = ' '.repeat(level * indent);

    if (obj === null || obj === undefined) {
      return 'null';
    }

    if (typeof obj === 'boolean') {
      return obj ? 'true' : 'false';
    }

    if (typeof obj === 'number') {
      return String(obj);
    }

    if (typeof obj === 'string') {
      // Quote if contains special chars
      if (/[:;{}\[\]@"\n\r#\/]/.test(obj) || obj.trim() !== obj) {
        return '"' + obj.replace(/"/g, '\\"') + '"';
      }
      return obj;
    }

    if (obj instanceof Date) {
      return '@datetime ' + obj.toISOString();
    }

    if (Array.isArray(obj)) {
      return obj.map(v => this.stringify(v, { indent, level: 0 })).join('; ');
    }

    if (typeof obj === 'object') {
      const entries = Object.entries(obj);
      if (entries.length === 0) return '{}';

      // Check if we should use inline or block
      const useInline = options.inline || entries.length <= 2;

      if (useInline && level > 0) {
        const pairs = entries.map(([k, v]) => {
          return `${k}: ${this.stringify(v, { indent, level: 0, inline: true })}`;
        }).join('; ');
        return `{${pairs}}`;
      }

      const lines = [];
      for (const [key, value] of entries) {
        if (typeof value === 'object' && value !== null && !Array.isArray(value) && !(value instanceof Date)) {
          const sub = this.stringify(value, { indent, level: level + 1 });
          if (sub.startsWith('{')) {
            lines.push(`${prefix}${key}: ${sub}`);
          } else {
            lines.push(`${prefix}${key}:`);
            lines.push(sub);
          }
        } else {
          lines.push(`${prefix}${key}: ${this.stringify(value, { indent, level: 0 })}`);
        }
      }
      return lines.join('\n');
    }

    return String(obj);
  }

  // ==================== INTERNAL METHODS ====================

  _preprocess(source) {
    let text = source.replace(/\r\n/g, '\n').replace(/\r/g, '\n');
    return this._stripComments(text);
  }

  /**
   * Strip '#' line comments and '/* *\/' block comments while ignoring
   * both inside single/double-quoted strings (so values like @color
   * "#ff0080" are not mistaken for the start of a comment).
   */
  _stripComments(text) {
    let result = '';
    let i = 0;
    let inQuote = null;

    while (i < text.length) {
      const ch = text[i];

      if (inQuote) {
        result += ch;
        if (ch === '\\' && i + 1 < text.length) {
          result += text[i + 1];
          i += 2;
          continue;
        }
        if (ch === inQuote) inQuote = null;
        i++;
        continue;
      }

      if (ch === '"' || ch === "'") {
        inQuote = ch;
        result += ch;
        i++;
        continue;
      }

      if (ch === '#') {
        const eol = text.indexOf('\n', i);
        i = eol === -1 ? text.length : eol;
        continue;
      }

      if (ch === '/' && text[i + 1] === '*') {
        const close = text.indexOf('*/', i + 2);
        i = close === -1 ? text.length : close + 2;
        continue;
      }

      result += ch;
      i++;
    }

    return result;
  }

  _splitLines(text) {
    return text.split('\n')
      .map((line, index) => {
        const trimmed = line.replace(/\t/g, '  '); // tabs to 2 spaces
        const indent = trimmed.match(/^(\s*)/)[0].length;
        const content = trimmed.trim();
        return { line: index + 1, indent, content };
      })
      .filter(l => l.content.length > 0);
  }

  _parseBlock(lines, startIdx, baseIndent) {
    const result = {};
    let i = startIdx;

    while (i < lines.length) {
      const line = lines[i];

      // Skip lines at same or lower indent than base (end of block)
      if (line.indent <= baseIndent) {
        break;
      }

      // Parse key: value
      const colonIdx = line.content.indexOf(':');
      if (colonIdx === -1) {
        throw new YANParseError(`Expected ':' on line ${line.line}: "${line.content}"`);
      }

      const key = line.content.substring(0, colonIdx).trim();
      let rawValue = line.content.substring(colonIdx + 1).trim();

      // Check for inline object start { }
      if (rawValue.startsWith('{')) {
        const { value, nextIdx } = this._parseInlineObject(lines, i, rawValue);
        result[key] = value;
        i = nextIdx;
        continue;
      }

      // Check if next line is indented more (block value)
      const nextLine = lines[i + 1];
      if (nextLine && nextLine.indent > line.indent && !rawValue) {
        // Block value follows
        const { result: blockValue, nextIdx } = this._parseBlock(lines, i + 1, line.indent);
        result[key] = blockValue;
        i = nextIdx;
        continue;
      }

      // Parse inline value
      result[key] = this._parseValue(rawValue, line.line);
      i++;
    }

    return { result, nextIdx: i };
  }

  _parseInlineObject(lines, startIdx, rawValue) {
    // Find matching closing brace
    let braceCount = 0;
    let content = '';
    let i = startIdx;
    let j = 0;

    // Collect all text until matching }
    while (i < lines.length) {
      const line = lines[i];
      const text = i === startIdx ? rawValue : line.content;

      for (let k = 0; k < text.length; k++) {
        const ch = text[k];
        if (ch === '{') braceCount++;
        else if (ch === '}') {
          braceCount--;
          if (braceCount === 0) {
            content += text.substring(0, k);
            // Parse the content inside braces
            const obj = this._parseInlinePairs(content.substring(1)); // skip first {
            return { value: obj, nextIdx: i + 1 };
          }
        }
      }

      content += text + ' ';
      i++;
    }

    throw new YANParseError(`Unclosed '{' starting near line ${lines[startIdx].line}`);
  }

  _parseInlinePairs(text) {
    const result = {};
    // Split by ; or , but respect quotes and braces
    const pairs = this._smartSplit(text, /[,;]/);

    for (const pair of pairs) {
      const trimmed = pair.trim();
      if (!trimmed) continue;

      const colonIdx = trimmed.indexOf(':');
      if (colonIdx === -1) continue;

      const key = trimmed.substring(0, colonIdx).trim();
      const value = trimmed.substring(colonIdx + 1).trim();
      result[key] = this._parseValue(value, 0);
    }

    return result;
  }

  _smartSplit(text, delimiter) {
    const parts = [];
    let current = '';
    let inQuotes = false;
    let braceDepth = 0;

    for (const ch of text) {
      if (ch === '"' && !inQuotes) {
        inQuotes = true;
        current += ch;
      } else if (ch === '"' && inQuotes) {
        inQuotes = false;
        current += ch;
      } else if (ch === '{' && !inQuotes) {
        braceDepth++;
        current += ch;
      } else if (ch === '}' && !inQuotes) {
        braceDepth--;
        current += ch;
      } else if (delimiter.test(ch) && !inQuotes && braceDepth === 0) {
        parts.push(current);
        current = '';
      } else {
        current += ch;
      }
    }

    if (current.trim()) parts.push(current);
    return parts;
  }

  _parseValue(raw, lineNum) {
    const value = raw.trim();

    if (value === '') return null;

    // Check for type hints @type value
    if (value.startsWith('@')) {
      return this._parseTypeHint(value, lineNum);
    }

    // Check for array (contains ; not in quotes)
    if (this._isArray(value)) {
      return this._parseArray(value, lineNum);
    }

    // Check for inline object
    if (value.startsWith('{')) {
      return this._parseInlinePairs(value);
    }

    // String (quoted)
    if (value.startsWith('"')) {
      return this._parseString(value, lineNum);
    }

    // Boolean
    const lower = value.toLowerCase();
    if (['true', 'yes', 'on'].includes(lower)) return true;
    if (['false', 'no', 'off'].includes(lower)) return false;

    // Null
    if (['null', 'nil', '_', '~'].includes(lower)) return null;

    // Number
    if (/^-?\d+(\.\d+)?([eE][+-]?\d+)?$/.test(value)) {
      return value.includes('.') ? parseFloat(value) : parseInt(value, 10);
    }

    // Unquoted string
    return value;
  }

  _isArray(text) {
    let inQuotes = false;
    for (const ch of text) {
      if (ch === '"') inQuotes = !inQuotes;
      else if (ch === ';' && !inQuotes) return true;
    }
    return false;
  }

  _parseArray(text, lineNum) {
    const items = this._smartSplit(text, /;/);
    return items.map(item => this._parseValue(item.trim(), lineNum)).filter(v => v !== undefined);
  }

  _parseString(text, lineNum) {
    if (!text.startsWith('"')) return text;
    if (!text.endsWith('"')) {
      throw new YANParseError(`Unclosed string on line ${lineNum}: ${text}`);
    }
    return text.slice(1, -1).replace(/\\"/g, '"');
  }

  _parseTypeHint(text, lineNum) {
    const spaceIdx = text.indexOf(' ');
    const type = spaceIdx === -1 ? text.slice(1) : text.slice(1, spaceIdx);
    const rawValue = spaceIdx === -1 ? '' : text.slice(spaceIdx + 1);

    switch (type) {
      case 'int':
        return parseInt(rawValue, 10);
      case 'float':
        return parseFloat(rawValue);
      case 'date':
        return { __type: 'date', __value: rawValue };
      case 'datetime':
        return { __type: 'datetime', __value: rawValue };
      case 'bool':
        return ['true', 'yes', 'on', '1'].includes(rawValue.toLowerCase());
      case 'hex':
        return { __type: 'hex', __value: rawValue };
      case 'base64':
        return { __type: 'base64', __value: rawValue };
      case 'uuid':
        return { __type: 'uuid', __value: rawValue };
      case 'url':
        return { __type: 'url', __value: rawValue };
      case 'regex':
        return new RegExp(rawValue);
      case 'bigint': {
        if (!/^-?\d+$/.test(rawValue)) {
          throw new YANParseError(`Invalid @bigint value on line ${lineNum}: "${rawValue}"`);
        }
        return { __type: 'bigint', __value: BigInt(rawValue).toString() };
      }
      case 'email': {
        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(rawValue)) {
          throw new YANParseError(`Invalid @email value on line ${lineNum}: "${rawValue}"`);
        }
        return { __type: 'email', __value: rawValue };
      }
      case 'ipv4': {
        const octets = rawValue.split('.');
        const valid = octets.length === 4 && octets.every(o => /^\d{1,3}$/.test(o) && Number(o) <= 255);
        if (!valid) {
          throw new YANParseError(`Invalid @ipv4 value on line ${lineNum}: "${rawValue}"`);
        }
        return { __type: 'ipv4', __value: rawValue };
      }
      case 'ipv6': {
        if (!/^[0-9a-fA-F:]+$/.test(rawValue) || !rawValue.includes(':')) {
          throw new YANParseError(`Invalid @ipv6 value on line ${lineNum}: "${rawValue}"`);
        }
        return { __type: 'ipv6', __value: rawValue };
      }
      case 'color': {
        let colorValue = rawValue.trim();
        if ((colorValue.startsWith('"') && colorValue.endsWith('"')) ||
            (colorValue.startsWith("'") && colorValue.endsWith("'"))) {
          colorValue = colorValue.slice(1, -1);
        }
        if (!/^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/.test(colorValue)) {
          throw new YANParseError(`Invalid @color value on line ${lineNum}: "${rawValue}" (note: hex colors must be quoted, e.g. @color "#ff0080", since unquoted '#' starts a comment)`);
        }
        return { __type: 'color', __value: colorValue };
      }
      case 'duration': {
        if (!/^-?(\d+(\.\d+)?(d|h|m|s|ms))+$/.test(rawValue)) {
          throw new YANParseError(`Invalid @duration value on line ${lineNum}: "${rawValue}"`);
        }
        return { __type: 'duration', __value: rawValue };
      }
      default:
        return { __type: type, __value: this._parseValue(rawValue, lineNum) };
    }
  }
}

class YANParseError extends Error {
  constructor(message) {
    super(message);
    this.name = 'YANParseError';
  }
}

// ==================== EXPORTS ====================

if (typeof module !== 'undefined' && module.exports) {
  module.exports = { YANParser, YANParseError };
}

if (typeof window !== 'undefined') {
  window.YANParser = YANParser;
  window.YANParseError = YANParseError;
}

// ==================== DEMO / TEST ====================

function demo() {
  const parser = new YANParser();

  const sample = `
# Configurasi aplikasi
app:
  name: "Super App"
  version: 1.2.0
  debug: off
  max_users: @int 1000

database:
  host: localhost
  port: 5432
  ssl: yes
  pool: {min: 5; max: 20}

features:
  auth: {enabled: yes; provider: google}
  cache: {enabled: no; ttl: 300}

metadata:
  created: @datetime 2026-06-27T13:00:00Z
  author: _
  tags: api; backend; v1

/* Multi-line
   comment here */
user:
  name: Budi
  umur: 25
  address: {
    city: jakarta
  }
  hobbies: makan; tidur; ngoding
  `;

  try {
    const result = parser.parse(sample);
    console.log("=== PARSED ===");
    console.log(JSON.stringify(result, null, 2));

    console.log("\n=== STRINGIFY BACK ===");
    console.log(parser.stringify(result));
  } catch (e) {
    console.error("Parse error:", e.message);
  }
}

// Run demo if executed directly
if (typeof require !== 'undefined' && require.main === module) {
  demo();
}
