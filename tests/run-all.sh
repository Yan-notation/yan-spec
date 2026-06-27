#!/bin/bash
set -e

echo "=========================================="
echo "YAN Golden Test Suite"
echo "=========================================="
echo ""

FAILED=0
PASSED=0

# Run JS tests
echo "Running JavaScript tests..."
cd implementations/js
if node test/test.js; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi
cd ../..

echo ""

# Run Python tests
echo "Running Python tests..."
cd implementations/py
if python3 test/test.py; then
    PASSED=$((PASSED + 1))
else
    FAILED=$((FAILED + 1))
fi
cd ../..

echo ""

# Build C implementation
echo "Building C implementation..."
make -C implementations/c >/dev/null 2>&1
if [ -x implementations/c/yan ]; then
    echo "  ✓ C build OK"
else
    echo "  ✗ C build FAILED"
    FAILED=$((FAILED + 1))
fi

echo ""

# Cross-language golden tests
echo "Running cross-language golden tests..."
for yan_file in tests/valid/**/*.yan; do
    json_file="${yan_file%.yan}.json"
    if [ ! -f "$json_file" ]; then
        echo "  SKIP: No JSON expected for $yan_file"
        continue
    fi

    echo "  Testing: $yan_file"

    # Parse with Python and compare
    python3 -c "
import sys, json
sys.path.insert(0, 'implementations/py/src')
from yan import parse
with open('$yan_file') as f:
    result = parse(f.read())
with open('$json_file') as f:
    expected = json.load(f)
if result == expected:
    print('    ✓ Python OK')
    sys.exit(0)
else:
    print('    ✗ Python MISMATCH')
    print('    Got:', result)
    print('    Expected:', expected)
    sys.exit(1)
" && PASSED=$((PASSED + 1)) || FAILED=$((FAILED + 1))

    # Parse with JS and compare
    node -e "
const { YANParser } = require('./implementations/js/src/yan-parser');
const fs = require('fs');
const parser = new YANParser();
const result = parser.parse(fs.readFileSync('$yan_file', 'utf8'));
const expected = JSON.parse(fs.readFileSync('$json_file', 'utf8'));
if (JSON.stringify(result) === JSON.stringify(expected)) {
    console.log('    ✓ JS OK');
    process.exit(0);
} else {
    console.log('    ✗ JS MISMATCH');
    console.log('    Got:', JSON.stringify(result));
    console.log('    Expected:', JSON.stringify(expected));
    process.exit(1);
}
" && PASSED=$((PASSED + 1)) || FAILED=$((FAILED + 1))

    # Parse with C and compare
    python3 -c "
import sys, json, subprocess
with open('$json_file') as f:
    expected = json.load(f)
result = subprocess.run(['./implementations/c/yan', '$yan_file'], capture_output=True, text=True)
try:
    actual = json.loads(result.stdout)
except Exception as e:
    print('    ✗ C PARSE ERROR:', e)
    sys.exit(1)
if actual == expected:
    print('    ✓ C OK')
    sys.exit(0)
else:
    print('    ✗ C MISMATCH')
    print('    Got:', actual)
    print('    Expected:', expected)
    sys.exit(1)
" && PASSED=$((PASSED + 1)) || FAILED=$((FAILED + 1))
done

echo ""
echo "=========================================="
echo "Results: $PASSED passed, $FAILED failed"
echo "=========================================="

if [ $FAILED -gt 0 ]; then
    exit 1
fi
