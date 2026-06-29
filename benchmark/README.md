# YAN Benchmark

Compares parse throughput of the reference JS implementation
(`implementations/js`) against the most popular JS parsing library for
each competing format: native `JSON.parse`, `js-yaml`, and `@iarna/toml`.

## Running it

```bash
cd benchmark
npm install
node benchmark.js
```

## Methodology

All four formats encode the *same* logical data (a small nested config:
app settings, a database block, feature flags, a 3-item array of server
objects, and a tags array — see `fixture.js`). Each parser runs 1,000
warmup iterations followed by 20,000 timed iterations; we report the
average per-parse time and operations/second.

## Results (example run)

Machine: GitHub-Actions-class Linux container, Node v22.

| Format | Avg parse | Ops/sec | vs JSON |
|--------|-----------|---------|---------|
| JSON   | 3.28 µs   | 304,832 | 1.00x   |
| TOML   | 40.66 µs  | 24,595  | 0.08x   |
| YAML   | 46.12 µs  | 21,682  | 0.07x   |
| YAN    | 53.96 µs  | 18,532  | 0.06x   |

**Honest takeaway: YAN is currently the slowest of the four**, including
slower than YAML and TOML. This isn't surprising and we're not going to
spin it as good news:

- `JSON.parse` is a native, highly-optimized engine built-in — nothing
  hand-written in JS will get close to it. This gap is expected and not
  very informative.
- `js-yaml` and `@iarna/toml` are mature, performance-tuned libraries
  with years of optimization work behind them.
- The reference YAN parser (`implementations/js/src/yan-parser.js`) is a
  straightforward hand-rolled recursive-descent parser with **no
  performance work done yet** — it re-scans strings character-by-character
  in several places (e.g. `_smartSplit`, `_stripComments`) where a
  single-pass tokenizer would be faster.

## Where the time likely goes

Based on the implementation as it stands:
1. `_stripComments` walks the entire source character-by-character before
   any real parsing starts (a full extra pass over the input).
2. `_smartSplit` is called repeatedly (once per inline object/array) and
   itself does a fresh character-by-character scan each time, with no
   reuse between calls.
3. Regex-based scalar detection (`_parseValue`) runs multiple `RegExp.test`
   calls per value rather than a single combined tokenizer pass.

None of this is fixed yet — this benchmark exists to make the current
state visible and honest, not to claim YAN is fast. If/when the parser
is optimized (e.g. a proper single-pass tokenizer), re-run this script
to see whether it actually moved the needle.

## Caveats

- Single fixture size/shape — a deeply nested or very large document
  could shift these numbers in either direction.
- Only the JS reference implementation is benchmarked here. Python, C,
  Rust, and Go implementations are not included and would need their
  own benchmark harnesses (C and Rust, being compiled, would likely
  show a very different picture than this JS-vs-JS-libraries comparison).
- Results depend on machine, Node version, and CPU load at the time of
  the run — treat the exact numbers as illustrative, not authoritative.
