#!/usr/bin/env node
'use strict';

const fs = require('fs');
const path = require('path');
const { YANParser } = require('../src/yan-parser');

const VERSION = require('../package.json').version;

function printUsage() {
  console.log(`yan — YAN (Yet Another Notation) command line tool

Usage:
  yan parse <file>              Parse a .yan file and print it as JSON
  yan fmt <file> [--write]      Reformat a .yan file to canonical style
  yan validate <file>           Check a .yan file for syntax errors
  yan --version                 Print version
  yan --help                    Show this help

Examples:
  yan parse config.yan
  yan fmt config.yan --write
  yan validate config.yan
`);
}

function readFileOrExit(file) {
  if (!fs.existsSync(file)) {
    console.error(`yan: cannot open '${file}': No such file`);
    process.exit(1);
  }
  return fs.readFileSync(file, 'utf8');
}

function cmdParse(file) {
  const source = readFileOrExit(file);
  const parser = new YANParser();
  try {
    const result = parser.parse(source);
    console.log(JSON.stringify(result, null, 2));
  } catch (err) {
    console.error(`yan: ${err.message}`);
    process.exit(1);
  }
}

function cmdFmt(file, write) {
  const source = readFileOrExit(file);
  const parser = new YANParser();
  try {
    const result = parser.parse(source);
    const formatted = parser.stringify(result);
    if (write) {
      fs.writeFileSync(file, formatted + '\n');
      console.log(`yan: formatted ${file}`);
    } else {
      console.log(formatted);
    }
  } catch (err) {
    console.error(`yan: ${err.message}`);
    process.exit(1);
  }
}

function cmdValidate(file) {
  const source = readFileOrExit(file);
  const parser = new YANParser();
  try {
    parser.parse(source);
    console.log(`✓ ${file} is valid YAN`);
  } catch (err) {
    console.error(`✗ ${file} is invalid: ${err.message}`);
    process.exit(1);
  }
}

function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args[0] === '--help' || args[0] === '-h') {
    printUsage();
    return;
  }

  if (args[0] === '--version' || args[0] === '-v') {
    console.log(`yan ${VERSION}`);
    return;
  }

  const [command, file, ...rest] = args;

  if (!file) {
    console.error(`yan: missing file argument for '${command}'`);
    printUsage();
    process.exit(1);
  }

  switch (command) {
    case 'parse':
      cmdParse(file);
      break;
    case 'fmt':
      cmdFmt(file, rest.includes('--write'));
      break;
    case 'validate':
      cmdValidate(file);
      break;
    default:
      console.error(`yan: unknown command '${command}'`);
      printUsage();
      process.exit(1);
  }
}

main();
