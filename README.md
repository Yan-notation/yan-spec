# YAN — Yet Another Notation

[![Spec](https://img.shields.io/badge/spec-v1.0-blue)](./SPEC.md)
[![License](https://img.shields.io/badge/license-CC0-green)](./LICENSE)

> A human-readable, human-writable, and machine-readable data interchange format.

## What is YAN?

YAN (Yet Another Notation) is a lightweight, text-based, language-independent data format designed to be easy for humans to read and write while remaining trivial for machines to parse and generate.

```yan
# config.yan
server:
  host: localhost
  port: @int 8080
  debug: off

database:
  driver: postgresql
  pool: {min: 5; max: 20}
```

## Features

- **No quotes on keys** — `name: Budi` instead of `"name": "Budi"`
- **Comments** — `# line` and `/* block */`
- **Flexible booleans** — `true`, `false`, `yes`, `no`, `on`, `off`
- **Type hints** — `@date 2026-06-27`, `@hex deadbeef`
- **Hybrid nesting** — indentation + inline braces
- **Array with semicolons** — `hobbies: makan; tidur; ngoding`

## Specification

- [English Specification (SPEC.md)](./SPEC.md)
- [Spesifikasi Bahasa Indonesia (SPEC-ID.md)](./SPEC-ID.md)

## Implementations

| Language | Package | Status |
|----------|---------|--------|
| JavaScript | `yan-parser` | ✅ Available |
| Python | `yan` | ✅ Available |

## Quick Start

### JavaScript
```bash
npm install yan-parser
```
```js
const { YANParser } = require('yan-parser');
const data = new YANParser().parse(`name: Budi\nage: 25`);
```

### Python
```bash
pip install yan
```
```python
import yan
data = yan.parse("name: Budi\nage: 25")
```

## Test Suite

Run the golden test suite:
```bash
./tests/run-all.sh
```

## Contributing

See [GitHub Issues](https://github.com/yan-notation/yan-spec/issues) for open tasks.

## License

[CC0 1.0 Universal](./LICENSE) — Public Domain.
