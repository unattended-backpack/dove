use crate::collector::*;
use solang_parser::parse;
use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};

#[test]
fn test_import_types() {
    // Test different import types
    let source = r#"// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

// Plain import
import "./utils/Math.sol";

// Import with symbols
import { SafeMath, Counter } from "./libraries/SafeMath.sol";

// Import with alias
import { SafeMath as SM, Counter as C } from "./libraries/AliasedMath.sol";

// Import all as alias
import * as console from "hardhat/console.sol";

contract Test {
    uint256 public value;
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new()
        .pragma(|p| {
            p.content("solidity")
                .version("^0.8.0")
                .leading("// SPDX-License-Identifier: MIT")
        })
        .import(|i| {
            i.path("./utils/Math.sol")
                .plain()
                .leading("// Plain import")
        })
        .import(|i| {
            i.path("./libraries/SafeMath.sol")
                .symbol("SafeMath", None)
                .symbol("Counter", None)
                .leading("// Import with symbols")
        })
        .import(|i| {
            i.path("./libraries/AliasedMath.sol")
                .symbol("SafeMath", Some("SM"))
                .symbol("Counter", Some("C"))
                .leading("// Import with alias")
        })
        .import(|i| {
            i.path("hardhat/console.sol")
                .global_as("console")
                .leading("// Import all as alias")
        })
        .contract("Test", |c| {
            c.variable("value", |v| {
                v.ty("uint256")
                    .visibility("public")
            })
        });

    expected.assert_matches(&collected);
}