#!/usr/bin/env node
'use strict';

const yaml = require('js-yaml');
const toml = require('@iarna/toml');
const { YANParser } = require('../implementations/js/src/yan-parser');
const { json, yaml: yamlStr, toml: tomlStr, yan: yanStr } = require('./fixture.js');

const ITERATIONS = 20000;
const WARMUP = 1000;

function bench(name, fn) {
  for (let i = 0; i < WARMUP; i++) fn();

  const start = process.hrtime.bigint();
  for (let i = 0; i < ITERATIONS; i++) fn();
  const end = process.hrtime.bigint();

  const totalMs = Number(end - start) / 1e6;
  const avgUs = (totalMs * 1000) / ITERATIONS;
  const opsPerSec = Math.round(ITERATIONS / (totalMs / 1000));

  return { name, totalMs, avgUs, opsPerSec };
}

function formatRow(r, baselineOpsPerSec) {
  const relative = baselineOpsPerSec ? (r.opsPerSec / baselineOpsPerSec).toFixed(2) + 'x' : '1.00x (baseline)';
  return `| ${r.name.padEnd(8)} | ${r.avgUs.toFixed(2).padStart(8)} µs | ${r.opsPerSec.toLocaleString().padStart(12)} | ${relative.padStart(10)} |`;
}

console.log(`Fixture sizes (bytes): JSON=${json.length}  YAML=${yamlStr.length}  TOML=${tomlStr.length}  YAN=${yanStr.length}\n`);
console.log(`Iterations per format: ${ITERATIONS.toLocaleString()} (after ${WARMUP} warmup runs)\n`);

const parser = new YANParser();

const results = [
  bench('JSON', () => JSON.parse(json)),
  bench('YAML', () => yaml.load(yamlStr)),
  bench('TOML', () => toml.parse(tomlStr)),
  bench('YAN', () => parser.parse(yanStr)),
];

const jsonOps = results[0].opsPerSec;

console.log('| Format   | Avg parse  | Ops/sec      | vs JSON    |');
console.log('|----------|------------|--------------|------------|');
for (const r of results) {
  console.log(formatRow(r, jsonOps));
}

console.log('\nNote: this benchmarks the reference JS implementation only, against');
console.log('the most popular JS libraries for each competing format (js-yaml,');
console.log('@iarna/toml). Results will vary by machine, Node version, and payload');
console.log('shape. Re-run locally with `node benchmark.js` to reproduce.');
