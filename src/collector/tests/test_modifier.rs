#[cfg(test)]
mod tests {
    use crate::collector::test_builder::{ExpectationBuilder, CommentExpectation};
    use crate::collector::collect_source_unit;
    use solang_parser::parse;
    
    #[test]
    fn test_modifier_collection() {
        let source = r#"
        contract Test {
            // Only owner modifier
            modifier onlyOwner() {
                require(msg.sender == owner);
                _;
            }
            
            // With parameters
            modifier costs(uint price) {
                require(msg.value >= price);
                _;
            }
            
            // Regular function
            function doSomething() public onlyOwner {
                // Implementation
            }
        }
        "#;
        
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        
        // Modifiers are collected as functions, so we test them as functions
        let expected = ExpectationBuilder::new()
            .contract("Test", |c| {
                c.function("onlyOwner", |f| {
                    f.leading("// Only owner modifier")
                        .is_modifier()
                        .body_statement(|s| s.nested_statement(|s| s).nested_statement(|s| s))  // require and _
                })
                .function("costs", |f| {
                    f.leading("// With parameters")
                        .is_modifier()
                        .parameter("price", |p| p.ty("uint256"))
                        .body_statement(|s| s.nested_statement(|s| s).nested_statement(|s| s))  // require and _
                })
                .function("doSomething", |f| {
                    f.leading("// Regular function")
                        .visibility("public")
                        .modifier("onlyOwner")
                        .body_statement(|s| s.nested_standalone_comment("// Implementation"))
                })
            });
            
        expected.assert_matches(&collected);
    }
}