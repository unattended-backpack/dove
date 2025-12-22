#[cfg(test)]
mod tests {
    use crate::collector::collect_source_unit;
    use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
    use solang_parser::parse;

    #[test]
    fn test_contract_inheritance_with_comments() {
        let source = r#"
        // Base contract
        contract Base {
            // Base function
            function baseFunc() public virtual {}
        }
        
        // Inherited contract
        contract Child is Base {
            // Override function
            function baseFunc() public override {}
        }
        
        // Multiple inheritance
        contract GrandChild is Base, Child {
            // Complex override
            function baseFunc() public override(Base, Child) {}
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("Base", |c| {
                c.leading("// Base contract").function("baseFunc", |f| {
                    f.leading("// Base function")
                        .visibility("public")
                        .is_virtual()
                        .body_statement(|s| s) // Empty body
                })
            })
            .contract("Child", |c| {
                c.leading("// Inherited contract")
                    .inherits("Base")
                    .function("baseFunc", |f| {
                        f.leading("// Override function")
                            .visibility("public")
                            .is_override()
                            .body_statement(|s| s) // Empty body
                    })
            })
            .contract("GrandChild", |c| {
                c.leading("// Multiple inheritance")
                    .inherits("Base")
                    .inherits("Child")
                    .function("baseFunc", |f| {
                        f.leading("// Complex override")
                            .visibility("public")
                            .overrides(&["Base", "Child"])
                            .body_statement(|s| s) // Empty body
                    })
            });
        expected.assert_matches(&collected);
    }

    #[test]
    fn test_override_virtual_modifiers() {
        let source = r#"
        contract Base {
            // Virtual getter
            uint256 public x;
            
            // Virtual function
            function foo() public virtual returns (uint256) {
                return 42;
            }
        }
        
        contract Derived is Base {
            // Override state variable
            uint256 public override x = 100;
            
            // Override with virtual
            function foo() public virtual override returns (uint256) {
                // Super call
                return super.foo() + 1;
            }
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("Base", |c| {
                c.variable("x", |v| {
                    v.leading("// Virtual getter")
                        .ty("uint256")
                        .visibility("public")
                })
                .function("foo", |f| {
                    f.leading("// Virtual function")
                        .visibility("public")
                        .is_virtual()
                        .returns("uint256")
                        .body_statement(|s| s.nested_statement(|s| s)) // return statement
                })
            })
            .contract("Derived", |c| {
                c.inherits("Base")
                    .variable("x", |v| {
                        v.leading("// Override state variable")
                            .ty("uint256")
                            .visibility("public")
                            .is_override()
                            .initial_value("100")
                    })
                    .function("foo", |f| {
                        f.leading("// Override with virtual")
                            .visibility("public")
                            .is_virtual()
                            .is_override()
                            .returns("uint256")
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Super call"))
                                // return statement with comment
                            })
                    })
            });
        expected.assert_matches(&collected);
    }

    #[test]
    fn test_complex_modifiers_with_parameters() {
        let source = r#"
        contract ModifierTest {
            // Complex modifier with multiple params
            modifier validRange(uint256 min, uint256 max, string memory errMsg) {
                // Check range
                require(msg.value >= min && msg.value <= max, errMsg);
                _;
            }
            
            // Using complex modifier
            function deposit() public payable validRange(1 ether, 10 ether, "Out of range") {
                // Deposit logic
            }
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new().contract("ModifierTest", |c| {
            c.function("validRange", |f| {
                f.leading("// Complex modifier with multiple params")
                    .is_modifier()
                    .parameter("min", |p| p.ty("uint256"))
                    .parameter("max", |p| p.ty("uint256"))
                    .parameter("errMsg", |p| p.ty("string").storage("memory"))
                    .body_statement(|s| {
                        s.nested_statement(|s| s.leading("// Check range")) // require statement
                            .nested_statement(|s| s) // placeholder _
                    })
            })
            .function("deposit", |f| {
                f.leading("// Using complex modifier")
                    .visibility("public")
                    .mutability("payable")
                    .modifier_with_args("validRange", &["1 ether", "10 ether", "\"Out of range\""])
                    .body_statement(|s| s.nested_standalone_comment("// Deposit logic"))
            })
        });
        expected.assert_matches(&collected);
    }
}