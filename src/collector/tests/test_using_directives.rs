#[cfg(test)]
mod tests {
    use crate::collector::test_builder::{ExpectationBuilder, CommentExpectation};
    use crate::collector::collect_source_unit;
    use solang_parser::parse;
    
    #[test]
    fn test_using_directive_with_library_and_type() {
        let source = r#"
        // Using for specific type
        using SafeMath for uint256;
        
        // Using for all types
        using StringUtils for *;
        
        contract Test {
            // Using inside contract
            using Address for address payable;
        }
        "#;
        
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        
        let expected = ExpectationBuilder::new()
            .using(|u| {
                u.library("SafeMath")
                    .for_type("uint256")
                    .leading("// Using for specific type")
            })
            .using(|u| {
                u.library("StringUtils")
                    .for_type("*")
                    .leading("// Using for all types")
            })
            .contract("Test", |c| {
                c.using_directive(|u| {
                    u.library("Address")
                        .for_type("address payable")
                        .leading("// Using inside contract")
                })
            });
            
        expected.assert_matches(&collected);
    }

    #[test]
    fn test_using_directives_advanced() {
        let source = r#"
        // Library for safe math
        library SafeMath {
            // Safe addition
            function add(uint256 a, uint256 b) internal pure returns (uint256) {
                uint256 c = a + b;
                require(c >= a, "Overflow");
                return c;
            }
        }
        
        // Library with operators
        library FixedPoint {
            // Fixed point type
            struct Fixed {
                uint256 value;
            }
            
            // Addition operator
            function add(Fixed memory a, Fixed memory b) internal pure returns (Fixed memory) {
                return Fixed(a.value + b.value);
            }
            
            // Multiplication operator
            function mul(Fixed memory a, uint256 b) internal pure returns (Fixed memory) {
                return Fixed(a.value * b);
            }
        }
        
        // Contract using libraries
        contract MathUser {
            // Using for basic type
            using SafeMath for uint256;
            
            // Using for custom type
            using FixedPoint for FixedPoint.Fixed;
            
            // Using with operators (0.8.19+)
            using {SafeMath.add as +} for uint256;
            
            // Multiple operators
            using {
                FixedPoint.add as +,  // Addition
                FixedPoint.mul as *   // Multiplication
            } for FixedPoint.Fixed;
            
            // Global using
            using SafeMath for uint256 global;
            
            // Function using library
            function calculate(uint256 x, uint256 y) public pure returns (uint256) {
                // Uses SafeMath.add
                return x.add(y);
            }
        }
        "#;
        
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        
        let expected = ExpectationBuilder::new()
            .contract("SafeMath", |c| {
                c.leading("// Library for safe math")
                    .contract_type("library")
                    .function("add", |f| {
                        f.leading("// Safe addition")
                            .visibility("internal")
                            .mutability("pure")
                            .returns("uint256")
                            .parameter("a", |p| p.ty("uint256"))
                            .parameter("b", |p| p.ty("uint256"))
                            .body_statement(|s| {
                                s.nested_statement(|s| s)
                                    .nested_statement(|s| s)
                                    .nested_statement(|s| s)
                            })
                    })
            })
            .contract("FixedPoint", |c| {
                c.leading("// Library with operators")
                    .contract_type("library")
                    .struct_def("Fixed", |s| {
                        s.leading("// Fixed point type")
                            .field("value", |f| f.ty("uint256"))
                    })
                    .function("add", |f| {
                        f.leading("// Addition operator")
                            .visibility("internal")
                            .mutability("pure")
                            .returns("Fixed")
                            .return_param(|r| r.storage("memory"))
                            .parameter("a", |p| p.ty("Fixed").storage("memory"))
                            .parameter("b", |p| p.ty("Fixed").storage("memory"))
                            .body_statement(|s| {
                                s.nested_statement(|s| s)
                            })
                    })
                    .function("mul", |f| {
                        f.leading("// Multiplication operator")
                            .visibility("internal")
                            .mutability("pure")
                            .returns("Fixed")
                            .return_param(|r| r.storage("memory"))
                            .parameter("a", |p| p.ty("Fixed").storage("memory"))
                            .parameter("b", |p| p.ty("uint256"))
                            .body_statement(|s| {
                                s.nested_statement(|s| s)
                            })
                    })
            })
            .contract("MathUser", |c| {
                c.leading("// Contract using libraries")
                    .using_directive(|u| {
                        u.leading("// Using for basic type")
                    })
                    .using_directive(|u| {
                        u.leading("// Using for custom type")
                    })
                    .using_directive(|u| {
                        u.leading("// Using with operators (0.8.19+)")
                    })
                    .using_directive(|u| {
                        u.leading("// Multiple operators")
                    })
                    .using_directive(|u| {
                        u.leading("// Global using")
                    })
                    .function("calculate", |f| {
                        f.leading("// Function using library")
                            .visibility("public")
                            .mutability("pure")
                            .returns("uint256")
                            .parameter("x", |p| p.ty("uint256"))
                            .parameter("y", |p| p.ty("uint256"))
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Uses SafeMath.add")
                                })
                            })
                    })
            });
            
        expected.assert_matches(&collected);
    }
}