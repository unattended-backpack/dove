use crate::collector::*;
use solang_parser::parse;
use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};

#[test]
fn test_collect_source_unit_simple_contract() {
    let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// This is a simple token contract
// It demonstrates basic functionality
contract SimpleToken {
    // Token name
    string public name = "Simple"; // Inline comment about name
    
    // Token total supply
    uint256 public totalSupply;
    
    // Transfer tokens to another address
    function transfer(address to, uint256 amount) public {
        // TODO: Add implementation
    }
}

// This is a standalone comment at the end
"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    // Use the builder pattern for cleaner assertions
    let expected = ExpectationBuilder::new()
        .pragma(|p| {
            p.content("solidity")
                .version("^0.8.0")
                .leading("// SPDX-License-Identifier: MIT")
        })
        .contract("SimpleToken", |c| {
            c.leading_many(vec![
                "// This is a simple token contract",
                "// It demonstrates basic functionality",
            ])
            .variable("name", |v| {
                v.ty("string")
                    .visibility("public")
                    .leading("// Token name")
                    .trailing("// Inline comment about name")
            })
            .variable("totalSupply", |v| {
                v.ty("uint256")
                    .visibility("public")
                    .leading("// Token total supply")
            })
            .function("transfer", |f| {
                f.visibility("public")
                    .parameter("to", |p| p.ty("address"))
                    .parameter("amount", |p| p.ty("uint256"))
                    .leading("// Transfer tokens to another address")
                    .body_statement(|s| s.nested_standalone_comment("// TODO: Add implementation"))
            })
        })
        .standalone_comment("// This is a standalone comment at the end");

    expected.assert_matches(&collected);
}

#[test]
fn test_contract_standalone_comments() {
    // Test that standalone comments within contracts are properly collected
    let source = r#"contract Test {
    uint256 public value;
    
    // This is a standalone comment between elements
    
    function setValue(uint256 newValue) public {
        value = newValue;
    }
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new().contract("Test", |c| {
        c.variable("value", |v| v.ty("uint256").visibility("public"))
            .function("setValue", |f| {
                f.visibility("public")
                    .parameter("newValue", |p| p.ty("uint256"))
                    .body_statement(|s| s.nested_statement(|s| s))
            })
            .standalone_comment("// This is a standalone comment between elements")
    });

    expected.assert_matches(&collected);
}

#[test]
fn test_todo_comment_with_statement() {
    // Test that TODO comments are properly collected when followed by a statement
    let source = r#"contract Test {
    function transfer(address to, uint256 amount) public {
        // TODO: Add balance check
        require(amount > 0, "Amount must be positive");
    }
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new().contract("Test", |c| {
        c.function("transfer", |f| {
            f.visibility("public")
                .parameter("to", |p| p.ty("address"))
                .parameter("amount", |p| p.ty("uint256"))
                .body_statement(|s| s.nested_statement(|s| s.leading("// TODO: Add balance check")))
        })
    });

    expected.assert_matches(&collected);
}

#[test]
fn test_nested_comment_association() {
    // This test ensures we properly associate comments with deeply nested structures
    // including standalone comments vs leading comments in various contexts
    let source = r#"// Top level comment
contract Nested {
    // Contract level variable comment
    uint256 state; // inline state comment
    
    // This is standalone inside contract
    
    // Function with nested complexity
    function complex(uint x) public { // function trailing
        // Top of function
        if (x > 0) { // if trailing
            // Inside if
            state = x; // assignment trailing
            // After assignment
        } // closing brace comment
        
        // Between if and else
        
        // Another standalone
    } // Function closing
    
    // After function
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new().contract("Nested", |c| {
        c.leading("// Top level comment")
            .variable("state", |v| {
                v.ty("uint256")
                    .leading("// Contract level variable comment")
                    .trailing("// inline state comment")
            })
            .function("complex", |f| {
                f.visibility("public")
                    .parameter("x", |p| p.ty("uint256"))
                    .leading("// Function with nested complexity")
                    .trailing("// Function closing") // This is actually after the function's closing brace
                    .body_statement(|s| {
                        s.nested_statement(|s| {
                            s.leading_many(vec![
                                "// function trailing", // This comes first, right after function opening brace
                                "// Top of function", // This comes second, right before the if statement
                            ])
                            .trailing_many(vec![
                                "// closing brace comment", // After the closing brace (comes first in collected order)
                                "// if trailing",           // After the if condition (comes second)
                            ])
                            .nested_statement(|s| {
                                s.leading("// Inside if").trailing("// assignment trailing")
                            })
                            .nested_standalone_comment("// After assignment")
                        })
                        .nested_standalone_comment("// Between if and else")
                        .nested_standalone_comment("// Another standalone")
                    })
            }) // Function has a body block with nested statement
            .standalone_comment("// This is standalone inside contract")
            .standalone_comment("// After function")
    });

    expected.assert_matches(&collected);
}