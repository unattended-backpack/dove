#!/bin/bash
# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
total_tests=0
passed_tests=0
failed_tests=0

echo -e "${BLUE}🐦 Testing dove peck...${NC}\n"

# Find all *_in.sol files
for input_file in *_in.sol; do
    # Skip if no matching files found
    [[ ! -f "$input_file" ]] && continue

    # Extract the base name (e.g., "A" from "A_in.sol")
    base_name="${input_file%_in.sol}"
    expected_file="${base_name}_out.sol"

    # Check if corresponding _out.sol file exists
    if [[ ! -f "$expected_file" ]]; then
        echo -e "${YELLOW}⚠️  WARNING: No matching output file for $input_file (expected: $expected_file)${NC}"
        continue
    fi

    total_tests=$((total_tests + 1))

    echo -e "${BLUE}Testing:${NC} $input_file → $expected_file"

    # Run peck and capture output
    peck_output=$(../target/debug/dove peck "$input_file" 2>/dev/null)

    # Compare with expected output
    if diff -q <(echo "$peck_output") "$expected_file" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ PASS${NC}"
        passed_tests=$((passed_tests + 1))
    else
        echo -e "${RED}❌ FAIL${NC}"
        failed_tests=$((failed_tests + 1))

        echo -e "${YELLOW}Expected vs Actual Differences:${NC}"
        echo "----------------------------------------"
        diff --color=always -u "$expected_file" <(echo "$peck_output")
        echo "----------------------------------------"
    fi
    echo
done

# Summary
echo -e "${BLUE}📊 Test Summary:${NC}"
echo -e "Total tests: $total_tests"
echo -e "${GREEN}Passed: $passed_tests${NC}"
if [[ $failed_tests -gt 0 ]]; then
    echo -e "${RED}Failed: $failed_tests${NC}"
else
    echo -e "${GREEN}Failed: $failed_tests${NC}"
fi

# Exit with appropriate code
if [[ $failed_tests -gt 0 ]]; then
    echo -e "\n${RED}Some tests failed!${NC}"
    exit 1
else
    echo -e "\n${GREEN}All tests passed!${NC}"
    exit 0
fi
