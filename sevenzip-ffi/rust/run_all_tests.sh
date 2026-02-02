#!/bin/bash
#
# Comprehensive Test Runner
# Runs all tests and generates summary
#

set -e

cd "$(dirname "$0")"

echo "=================================================="
echo "sevenzip-ffi Comprehensive Test Suite"
echo "=================================================="
echo ""
echo "Platform: $(uname -s) $(uname -m)"
echo "Date: $(date)"
echo ""

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}→ Running unit tests...${NC}"
UNIT_RESULT=$(cargo test --lib --release 2>&1)
UNIT_COUNT=$(echo "$UNIT_RESULT" | grep -E "test result: ok\." | head -1 | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')
echo -e "${GREEN}✓ $UNIT_COUNT unit tests passed${NC}"
echo ""

echo -e "${BLUE}→ Running integration tests...${NC}"
INTEGRATION_RESULT=$(cargo test --test integration_tests --release 2>&1)
INTEGRATION_COUNT=$(echo "$INTEGRATION_RESULT" | grep -E "test result: ok\." | head -1 | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')
echo -e "${GREEN}✓ $INTEGRATION_COUNT integration tests passed${NC}"
echo ""

echo -e "${BLUE}→ Running documentation tests...${NC}"
DOC_RESULT=$(cargo test --doc --release 2>&1)
DOC_COUNT=$(echo "$DOC_RESULT" | grep -E "test result: ok\." | tail -1 | grep -oE '[0-9]+ passed' | grep -oE '[0-9]+')
echo -e "${GREEN}✓ $DOC_COUNT documentation tests passed${NC}"
echo ""

TOTAL=$((UNIT_COUNT + INTEGRATION_COUNT + DOC_COUNT))

echo "=================================================="
echo -e "${GREEN}✓ ALL TESTS PASSED${NC}"
echo ""
echo "Summary:"
echo "  Unit tests:        $UNIT_COUNT"
echo "  Integration tests: $INTEGRATION_COUNT"
echo "  Doc tests:         $DOC_COUNT"
echo "  ───────────────────────"
echo "  TOTAL:             $TOTAL"
echo ""
echo "=================================================="
echo ""
echo "Optional: Run benchmarks with:"
echo "  cargo bench"
echo ""
echo "Or run quick benchmarks with:"
echo "  ./quick_bench.sh"
echo "=================================================="
