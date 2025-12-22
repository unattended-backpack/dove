use crate::collector::*;
use solang_parser::parse;
use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};

#[test]
fn test_import_path_verification() {
    // Test that import paths are correctly verified
    let source = r#"pragma solidity ^0.8.0;

import "contracts/token/ERC20.sol";
import "./interfaces/IERC20.sol";
import "../lib/SafeMath.sol";

contract Test {}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new()
        .pragma(|p| p.content("solidity").version("^0.8.0"))
        .import(|i| i.path("contracts/token/ERC20.sol").plain())
        .import(|i| i.path("./interfaces/IERC20.sol").plain())
        .import(|i| i.path("../lib/SafeMath.sol").plain())
        .contract("Test", |c| c);

    expected.assert_matches(&collected);
}

#[test]
fn test_import_symbols_verification() {
    // Test that import symbols are correctly verified
    let source = r#"pragma solidity ^0.8.0;

import { ERC20, IERC20 } from "contracts/token/ERC20.sol";
import { SafeMath as SM } from "./lib/SafeMath.sol";
import * as Utils from "./utils/Utils.sol";

contract Test {}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new()
        .pragma(|p| p.content("solidity").version("^0.8.0"))
        .import(|i| {
            i.path("contracts/token/ERC20.sol")
                .symbol("ERC20", None)
                .symbol("IERC20", None)
        })
        .import(|i| {
            i.path("./lib/SafeMath.sol")
                .symbol("SafeMath", Some("SM"))
        })
        .import(|i| {
            i.path("./utils/Utils.sol")
                .global_as("Utils")
        })
        .contract("Test", |c| c);

    expected.assert_matches(&collected);
}

#[test]
fn test_import_with_comments() {
    // Test that import comments are correctly associated
    let source = r#"pragma solidity ^0.8.0;

// Core token implementation
import "contracts/token/ERC20.sol";

// Token interface
// Multiple lines
import { IERC20 } from "./interfaces/IERC20.sol"; // inline comment

contract Test {}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new()
        .pragma(|p| p.content("solidity").version("^0.8.0"))
        .import(|i| {
            i.path("contracts/token/ERC20.sol")
                .plain()
                .leading("// Core token implementation")
        })
        .import(|i| {
            i.path("./interfaces/IERC20.sol")
                .symbol("IERC20", None)
                .leading_many(vec!["// Token interface", "// Multiple lines"])
                .trailing("// inline comment")
        })
        .contract("Test", |c| c);

    expected.assert_matches(&collected);
}