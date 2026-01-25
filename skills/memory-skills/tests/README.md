# Memory Skills Test Suite

This test suite uses **behavioral observation testing** to verify skill triggering and command execution, since AI skill activation is probabilistic.

## Test Categories

### 1. Skill Triggering Tests (`skill-triggering/`)

Tests whether natural language prompts correctly trigger the `memory-skills` skill.

```bash
./skill-triggering/run-all.sh
```

**Prompts tested:**
- `remember-chinese.txt` - Chinese remember trigger
- `remember-english.txt` - English remember trigger
- `recall-chinese.txt` - Chinese recall trigger
- `recall-english.txt` - English recall trigger
- `save-preference.txt` - Preference saving trigger
- `project-context.txt` - Project context trigger

### 2. Command Tests (`commands/`)

Tests `/remember` and `/recall` command execution.

```bash
./commands/run-all.sh
```

**Commands tested:**
- `remember-global.txt` - Global scope remember
- `remember-project.txt` - Project scope remember
- `remember-auto.txt` - Auto-detect scope
- `recall-topic.txt` - Recall with topic
- `recall-empty.txt` - Recall non-existent topic

### 3. Integration Tests (`integration/`)

End-to-end testing of remember → recall workflow.

```bash
./integration/test-remember-recall.sh
```

## Running All Tests

```bash
# Full test suite
./run-all.sh

# Quick mode (skill triggering only)
./run-all.sh --quick
```

## Test Output

All test results are saved to:
```
/tmp/memory-skills-tests/{timestamp}/
├── skill-triggering/
├── commands/
└── integration/
```

## Adding New Tests

1. Create a prompt file in the appropriate `prompts/` directory
2. Update the `run-all.sh` script to include the new test
3. Run the test to verify it passes

## Test Patterns

### Skill Triggering Verification
```bash
# Check if skill was triggered in JSON output
grep -q '"skill":"memory-skills"' claude-output.json && echo "PASS"
```

### Command Output Verification
```bash
# Check for expected patterns in output
grep -q "已保存" claude-output.txt && echo "PASS"
```
