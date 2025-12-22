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

echo -e "${BLUE}🔧 Testing dove...${NC}\n"

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
    
    # Run formatter and capture output (first pass)
    first_pass_output=$(../target/debug/dove preen "$input_file" 2>/dev/null)

    # Save first pass output to temporary file for second formatting pass
    temp_file_1=$(mktemp "${base_name}_temp1_XXXXXX.sol")
    echo "$first_pass_output" > "$temp_file_1"

    # Run formatter on the first pass output (second pass)
    second_pass_output=$(../target/debug/dove preen "$temp_file_1" 2>/dev/null)

    # Save second pass output to temporary file for third formatting pass
    temp_file_2=$(mktemp "${base_name}_temp2_XXXXXX.sol")
    echo "$second_pass_output" > "$temp_file_2"

    # Run formatter on the second pass output (third pass)
    third_pass_output=$(../target/debug/dove preen "$temp_file_2" 2>/dev/null)
    
    # Clean up temp files
    rm -f "$temp_file_1" "$temp_file_2"
    
    # First test: Compare first pass with expected output
    first_pass_matches=false
    if diff -q <(echo "$first_pass_output") "$expected_file" >/dev/null 2>&1; then
        first_pass_matches=true
    fi
    
    # Second test: Check formatter stability (second pass should equal first pass)
    second_stability_passes=false
    if diff -q <(echo "$first_pass_output") <(echo "$second_pass_output") >/dev/null 2>&1; then
        second_stability_passes=true
    fi
    
    # Third test: Check formatter stability (third pass should equal second pass)
    third_stability_passes=false
    if diff -q <(echo "$second_pass_output") <(echo "$third_pass_output") >/dev/null 2>&1; then
        third_stability_passes=true
    fi
    
    # Overall result
    if [[ "$first_pass_matches" == true && "$second_stability_passes" == true && "$third_stability_passes" == true ]]; then
        echo -e "${GREEN}✅ PASS${NC} (format matches expected + formatter is stable across 3 passes)"
        passed_tests=$((passed_tests + 1))
    else
        echo -e "${RED}❌ FAIL${NC}"
        failed_tests=$((failed_tests + 1))
        
        if [[ "$first_pass_matches" == false ]]; then
            echo -e "${YELLOW}Expected vs First Pass Differences:${NC}"
            echo "----------------------------------------"
            diff --color=always -u "$expected_file" <(echo "$first_pass_output")
            echo "----------------------------------------"
        fi
        
        if [[ "$second_stability_passes" == false ]]; then
            echo -e "${YELLOW}Stability Test Failed - First Pass vs Second Pass Differences:${NC}"
            echo "----------------------------------------"
            diff --color=always -u <(echo "$first_pass_output") <(echo "$second_pass_output") | head -20
            echo "----------------------------------------"
        fi
        
        if [[ "$third_stability_passes" == false ]]; then
            echo -e "${YELLOW}Stability Test Failed - Second Pass vs Third Pass Differences:${NC}"
            echo "----------------------------------------"
            diff --color=always -u <(echo "$second_pass_output") <(echo "$third_pass_output") | head -20
            echo "----------------------------------------"
        fi
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
