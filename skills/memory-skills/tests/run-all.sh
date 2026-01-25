#!/bin/bash

# Memory Skills - Test Runner
# Usage: ./run-all.sh [--quick]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="/tmp/memory-skills-tests/${TIMESTAMP}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "================================================"
echo "Memory Skills Test Suite"
echo "================================================"
echo "Output directory: ${OUTPUT_DIR}"
echo ""

mkdir -p "${OUTPUT_DIR}"

QUICK_MODE=false
if [[ "$1" == "--quick" ]]; then
    QUICK_MODE=true
    echo -e "${YELLOW}Running in quick mode (skill triggering only)${NC}"
    echo ""
fi

PASS_COUNT=0
FAIL_COUNT=0

# Function to run a test
run_test() {
    local test_name=$1
    local prompt_file=$2
    local output_subdir=$3
    local expected_pattern=$4
    
    local test_output_dir="${OUTPUT_DIR}/${output_subdir}"
    mkdir -p "${test_output_dir}"
    
    echo -n "Testing: ${test_name}... "
    
    # Copy prompt
    cp "${prompt_file}" "${test_output_dir}/prompt.txt"
    
    # Run Claude (with timeout)
    if timeout 120 claude -p "$(cat "${prompt_file}")" \
        --dangerously-skip-permissions \
        --max-turns 3 \
        --output-format stream-json \
        > "${test_output_dir}/claude-output.json" 2>&1; then
        
        # Check for expected pattern
        if grep -q "${expected_pattern}" "${test_output_dir}/claude-output.json"; then
            echo -e "${GREEN}PASS${NC}"
            ((PASS_COUNT++))
        else
            echo -e "${RED}FAIL${NC} (pattern not found)"
            ((FAIL_COUNT++))
        fi
    else
        echo -e "${RED}FAIL${NC} (timeout or error)"
        ((FAIL_COUNT++))
    fi
}

# Skill Triggering Tests
echo ""
echo "--- Skill Triggering Tests ---"
for prompt in "${SCRIPT_DIR}/skill-triggering/prompts"/*.txt; do
    if [[ -f "$prompt" ]]; then
        test_name=$(basename "$prompt" .txt)
        run_test "$test_name" "$prompt" "skill-triggering/${test_name}" '"skill"'
    fi
done

# Command Tests (skip in quick mode)
if [[ "$QUICK_MODE" == false ]]; then
    echo ""
    echo "--- Command Tests ---"
    for prompt in "${SCRIPT_DIR}/commands/prompts"/*.txt; do
        if [[ -f "$prompt" ]]; then
            test_name=$(basename "$prompt" .txt)
            run_test "$test_name" "$prompt" "commands/${test_name}" '已保存\|saved\|Found\|找到\|没有找到'
        fi
    done
fi

# Summary
echo ""
echo "================================================"
echo "Test Summary"
echo "================================================"
echo -e "Passed: ${GREEN}${PASS_COUNT}${NC}"
echo -e "Failed: ${RED}${FAIL_COUNT}${NC}"
echo ""
echo "Full output: ${OUTPUT_DIR}"

if [[ $FAIL_COUNT -gt 0 ]]; then
    exit 1
fi
