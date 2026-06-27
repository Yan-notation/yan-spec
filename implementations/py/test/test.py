"""Tests for YAN Python parser."""

import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../src"))

from yan import parse, stringify, YANParser, YANParseError


def test(name, actual, expected):
    a = str(actual)
    b = str(expected)
    if a == b:
        print(f"  ✓ {name}")
        return True
    else:
        print(f"  ✗ {name}")
        print(f"    Expected: {b}")
        print(f"    Got:      {a}")
        return False


passed = 0
failed = 0

print("Running YAN Python Parser Tests...\n")

# Primitives
if test("primitive string", parse("name: Budi"), {"name": "Budi"}): passed += 1
else: failed += 1

if test("primitive number", parse("age: 25"), {"age": 25}): passed += 1
else: failed += 1

if test("primitive float", parse("pi: 3.14"), {"pi": 3.14}): passed += 1
else: failed += 1

if test("primitive boolean true", parse("active: true"), {"active": True}): passed += 1
else: failed += 1

if test("primitive boolean yes", parse("active: yes"), {"active": True}): passed += 1
else: failed += 1

if test("primitive boolean off", parse("debug: off"), {"debug": False}): passed += 1
else: failed += 1

if test("primitive null", parse("data: null"), {"data": None}): passed += 1
else: failed += 1

if test("primitive null underscore", parse("data: _"), {"data": None}): passed += 1
else: failed += 1

if test("quoted string", parse('msg: "hello world"'), {"msg": "hello world"}): passed += 1
else: failed += 1

# Array
if test("array semicolon", parse("tags: a; b; c"), {"tags": ["a", "b", "c"]}): passed += 1
else: failed += 1

# Inline object
if test("inline object", parse("cfg: {host: localhost; port: 80}"), {"cfg": {"host": "localhost", "port": 80}}): passed += 1
else: failed += 1

# Block object
if test("block object", parse("person:\n  name: Budi\n  age: 25"), {"person": {"name": "Budi", "age": 25}}): passed += 1
else: failed += 1

# Nested block
if test("nested block", parse("a:\n  b:\n    c: 1"), {"a": {"b": {"c": 1}}}): passed += 1
else: failed += 1

# Comments
if test("line comment", parse("# comment\nname: Budi"), {"name": "Budi"}): passed += 1
else: failed += 1

if test("block comment", parse("/* multi\nline */\nname: Budi"), {"name": "Budi"}): passed += 1
else: failed += 1

# Type hints
if test("type hint @int", parse("n: @int 42"), {"n": 42}): passed += 1
else: failed += 1

if test("type hint @float", parse("n: @float 3.14"), {"n": 3.14}): passed += 1
else: failed += 1

if test("type hint @bool", parse("b: @bool yes"), {"b": True}): passed += 1
else: failed += 1

# Trailing separators
if test("trailing semicolon array", parse("arr: a; b;"), {"arr": ["a", "b"]}): passed += 1
else: failed += 1

if test("trailing semicolon inline", parse("obj: {a: 1; b: 2;}"), {"obj": {"a": 1, "b": 2}}): passed += 1
else: failed += 1

# Full document
full_doc = '''
app:
  name: "Super App"
  version: 1.2.0
  debug: off

database:
  host: localhost
  port: 5432
  ssl: yes
  pool: {min: 5; max: 20}
'''
full_result = parse(full_doc)
full_ok = (
    full_result["app"]["name"] == "Super App" and
    full_result["app"]["debug"] == False and
    full_result["database"]["pool"] == {"min": 5, "max": 20}
)
if test("full document", full_ok, True): passed += 1
else: failed += 1

# Stringify roundtrip
obj = {"name": "Budi", "age": 25, "active": True}
yan_str = stringify(obj)
back = parse(yan_str)
if test("stringify roundtrip", back, obj): passed += 1
else: failed += 1

print(f"\n{passed} passed, {failed} failed")
sys.exit(1 if failed > 0 else 0)
