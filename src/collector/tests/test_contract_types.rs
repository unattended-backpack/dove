use crate::collector::*;
use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
use solang_parser::parse;

#[test]
fn test_contract_types_and_inheritance() {
    // Test different contract types and inheritance
    let source = r#"// Base interface
interface IBase {
    function baseMethod() external;
}

// Library contract
library MathLib {
    function add(uint a, uint b) internal pure returns (uint) {
        return a + b;
    }
}

// Contract with inheritance
contract Implementation is IBase {
    function baseMethod() external override {
        // Implementation
    }
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    let expected = ExpectationBuilder::new()
        .contract("IBase", |c| {
            c.contract_type("interface")
                .leading("// Base interface")
                .function("baseMethod", |f| f.visibility("external"))
        })
        .contract("MathLib", |c| {
            c.contract_type("library")
                .leading("// Library contract")
                .function("add", |f| {
                    f.visibility("internal")
                        .mutability("pure")
                        .parameter("a", |p| p.ty("uint256"))
                        .parameter("b", |p| p.ty("uint256"))
                        .body_statement(|s| s.nested_statement(|s| s))
                })
        })
        .contract("Implementation", |c| {
            c.contract_type("contract")
                .inherits("IBase")
                .leading("// Contract with inheritance")
                .function("baseMethod", |f| {
                    f.visibility("external")
                        .is_override()
                        .body_statement(|s| s.nested_standalone_comment("// Implementation"))
                })
        });

    expected.assert_matches(&collected);
}
