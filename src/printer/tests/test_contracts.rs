use crate::collector::collect_source_unit;
use crate::ir_builder::build_ir;
use crate::printer::Printer;
use solang_parser::parse;
use std::fs;
use std::path::Path;

/// Helper to reformat Solidity source code through the full pipeline.
fn reformat(source: &str) -> String {
    let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
    let collected = collect_source_unit(&source_unit, &comments, source);
    let ir = build_ir(&collected);
    let mut printer = Printer::new();
    printer.print(&ir)
}

/// Helper to read a testbench file.
fn read_testbench(filename: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("testbench")
        .join(filename);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e))
}

/// Assert that the printed and formatted output is as expexted.
pub fn assert_print_match(expected: String, actual: String) {
    if actual != expected {
        // Print a diff-like output for debugging
        println!("=== EXPECTED ===");
        println!("{}", expected);
        println!("=== ACTUAL ===");
        println!("{}", actual);
        println!("=== END ===");

        // Find first difference
        let expected_lines: Vec<&str> = expected.lines().collect();
        let actual_lines: Vec<&str> = actual.lines().collect();

        for (i, (exp, act)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
            if exp != act {
                println!("First difference at line {}:", i + 1);
                println!("  expected: {}", exp);
                println!("  actual:   {}", act);
                break;
            }
        }

        if expected_lines.len() != actual_lines.len() {
            println!(
                "Line count differs: expected {}, actual {}",
                expected_lines.len(),
                actual_lines.len()
            );
        }

        panic!("Output does not match expected");
    }
}

#[test]
fn test_a() {
    let input = read_testbench("A_in.sol");
    let expected = read_testbench("A_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_b() {
    let input = read_testbench("B_in.sol");
    let expected = read_testbench("B_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_c() {
    let input = read_testbench("C_in.sol");
    let expected = read_testbench("C_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_d() {
    let input = read_testbench("D_in.sol");
    let expected = read_testbench("D_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_e() {
    let input = read_testbench("E_in.sol");
    let expected = read_testbench("E_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_f() {
    let input = read_testbench("F_in.sol");
    let expected = read_testbench("F_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_g() {
    let input = read_testbench("G_in.sol");
    let expected = read_testbench("G_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_h() {
    let input = read_testbench("H_in.sol");
    let expected = read_testbench("H_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_i() {
    let input = read_testbench("I_in.sol");
    let expected = read_testbench("I_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_j() {
    let input = read_testbench("J_in.sol");
    let expected = read_testbench("J_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}

#[test]
fn test_k() {
    let input = read_testbench("K_in.sol");
    let expected = read_testbench("K_out.sol");
    let actual = reformat(&input);
    assert_print_match(expected, actual);
}
