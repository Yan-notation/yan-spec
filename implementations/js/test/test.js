const { YANParser } = require('../src/yan-parser');
const fs = require('fs');
const path = require('path');

const parser = new YANParser();
let passed = 0, failed = 0;

function test(name, fn) {
  try {
    fn();
    console.log(`  ✓ ${name}`);
    passed++;
  } catch (e) {
    console.log(`  ✗ ${name}: ${e.message}`);
    failed++;
  }
}

function assertEqual(actual, expected) {
  const a = JSON.stringify(actual);
  const b = JSON.stringify(expected);
  if (a !== b) throw new Error(`Expected ${b}, got ${a}`);
}

console.log('Running YAN Parser Tests...\n');

test('primitive string', () => {
  assertEqual(parser.parse('name: Budi'), { name: 'Budi' });
});

test('primitive number', () => {
  assertEqual(parser.parse('age: 25'), { age: 25 });
});

test('primitive float', () => {
  assertEqual(parser.parse('pi: 3.14'), { pi: 3.14 });
});

test('primitive boolean true', () => {
  assertEqual(parser.parse('active: true'), { active: true });
});

test('primitive boolean yes', () => {
  assertEqual(parser.parse('active: yes'), { active: true });
});

test('primitive boolean off', () => {
  assertEqual(parser.parse('debug: off'), { debug: false });
});

test('primitive null', () => {
  assertEqual(parser.parse('data: null'), { data: null });
});

test('primitive null underscore', () => {
  assertEqual(parser.parse('data: _'), { data: null });
});

test('quoted string', () => {
  assertEqual(parser.parse('msg: "hello world"'), { msg: 'hello world' });
});

test('array semicolon', () => {
  assertEqual(parser.parse('tags: a; b; c'), { tags: ['a', 'b', 'c'] });
});

test('inline object', () => {
  assertEqual(parser.parse('cfg: {host: localhost; port: 80}'), { cfg: { host: 'localhost', port: 80 } });
});

test('block object', () => {
  const result = parser.parse('person:\n  name: Budi\n  age: 25');
  assertEqual(result, { person: { name: 'Budi', age: 25 } });
});

test('nested block', () => {
  const result = parser.parse('a:\n  b:\n    c: 1');
  assertEqual(result, { a: { b: { c: 1 } } });
});

test('line comment', () => {
  assertEqual(parser.parse('# comment\nname: Budi'), { name: 'Budi' });
});

test('block comment', () => {
  assertEqual(parser.parse('/* multi\nline */\nname: Budi'), { name: 'Budi' });
});

test('type hint @int', () => {
  assertEqual(parser.parse('n: @int 42'), { n: 42 });
});

test('type hint @float', () => {
  assertEqual(parser.parse('n: @float 3.14'), { n: 3.14 });
});

test('type hint @bool', () => {
  assertEqual(parser.parse('b: @bool yes'), { b: true });
});

test('trailing semicolon in array', () => {
  assertEqual(parser.parse('arr: a; b;'), { arr: ['a', 'b'] });
});

test('trailing semicolon in inline object', () => {
  assertEqual(parser.parse('obj: {a: 1; b: 2;}'), { obj: { a: 1, b: 2 } });
});

test('full document', () => {
  const doc = `
app:
  name: "Super App"
  version: 1.2.0
  debug: off

database:
  host: localhost
  port: 5432
  ssl: yes
  pool: {min: 5; max: 20}
`;
  const result = parser.parse(doc);
  assertEqual(result.app.name, 'Super App');
  assertEqual(result.app.debug, false);
  assertEqual(result.database.pool, { min: 5, max: 20 });
});

test('stringify roundtrip', () => {
  const obj = { name: 'Budi', age: 25, active: true };
  const yan = parser.stringify(obj);
  const back = parser.parse(yan);
  assertEqual(back, obj);
});

console.log(`\n${passed} passed, ${failed} failed`);
process.exit(failed > 0 ? 1 : 0);
