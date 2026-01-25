#!/bin/bash

# Integration Test: Remember → Recall workflow
# Tests that remembered content can be recalled

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="/tmp/memory-skills-tests/${TIMESTAMP}/integration"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo "================================================"
echo "Integration Test: Remember → Recall"
echo "================================================"
echo ""

mkdir -p "${OUTPUT_DIR}"

# Test content with unique identifier
TEST_ID="integration_test_${TIMESTAMP}"
TEST_CONTENT="Integration test marker: ${TEST_ID}"

echo "Step 1: Remember test content..."
claude -p "/remember ${TEST_CONTENT}" \
    --dangerously-skip-permissions \
    --max-turns 3 \
    --output-format stream-json \
    > "${OUTPUT_DIR}/remember.json" 2>&1

echo "Step 2: Recall test content..."
claude -p "/recall ${TEST_ID}" \
    --dangerously-skip-permissions \
    --max-turns 3 \
    --output-format stream-json \
    > "${OUTPUT_DIR}/recall.json" 2>&1

# Verify recall found the content
if grep -q "${TEST_ID}" "${OUTPUT_DIR}/recall.json"; then
    echo -e "${GREEN}PASS${NC}: Content was successfully remembered and recalled"
else
    echo -e "${RED}FAIL${NC}: Content was not found in recall"
    exit 1
fi

echo ""
echo "Output: ${OUTPUT_DIR}"
