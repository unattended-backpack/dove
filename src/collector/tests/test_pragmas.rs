use crate::collector::*;
use solang_parser::parse;
use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};

#[test]
fn test_all_pragma_types() {
    // Test all three types of pragma directives
    let source = r#"// Version pragma
pragma solidity ^0.8.0;

// Single identifier pragma
pragma experimental SMTChecker;

// Double identifier pragma
pragma abicoder v2;

// String literal pragma (custom example)
pragma arbitrary "someStringValue";

contract Test {
    uint256 public value;
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    // Use the builder pattern for cleaner assertions
    let expected = ExpectationBuilder::new()
        // Version pragma
        .pragma(|p| {
            p.content("solidity")
                .version("^0.8.0")
                .leading("// Version pragma")
        })
        // Single identifier pragma
        .pragma(|p| {
            p.content("experimental")
                .second_identifier("SMTChecker")
                .leading("// Single identifier pragma")
        })
        // Double identifier pragma
        .pragma(|p| {
            p.content("abicoder")
                .second_identifier("v2")
                .leading("// Double identifier pragma")
        })
        // String literal pragma
        .pragma(|p| {
            p.content("arbitrary")
                .string_value("someStringValue")
                .leading("// String literal pragma (custom example)")
        })
        .contract("Test", |c| {
            c.variable("value", |v| v.ty("uint256").visibility("public"))
        });

    expected.assert_matches(&collected);
}