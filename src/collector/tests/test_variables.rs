#[cfg(test)]
mod tests {
    use crate::collector::collect_source_unit;
    use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
    use solang_parser::parse;

    #[test]
    fn test_variable_types_and_attributes() {
        // Test that variable types and attributes are properly verified
        let source = r#"contract VariableTest {
    // Simple types
    uint256 public totalSupply;
    address private owner;
    bool internal paused;
    
    // With initial values
    string public constant NAME = "Test Token";
    uint8 public immutable DECIMALS = 18;
}"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("VariableTest", |c| {
            c.variable("totalSupply", |v| {
                v.ty("uint256")
                    .visibility("public")
                    .leading("// Simple types")
            })
            .variable("owner", |v| v.ty("address").visibility("private"))
            .variable("paused", |v| v.ty("bool").visibility("internal"))
            .variable("NAME", |v| {
                v.ty("string")
                    .visibility("public")
                    .constant()
                    .leading("// With initial values")
            })
            .variable("DECIMALS", |v| {
                v.ty("uint8").visibility("public").immutable()
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_mapping_declarations_with_comments() {
        let source = r#"
        contract MappingTest {
            // Simple mapping
            mapping(address => uint256) public balances;
            
            // Nested mapping
            mapping(address => mapping(uint256 => bool)) public approvals;
            
            // Complex mapping with struct
            struct UserInfo {
                string name;
                uint256 age;
            }
            
            // Mapping to struct
            mapping(address => UserInfo) public users;
            
            // Mapping in function param
            function processMapping(
                // Mapping parameter
                mapping(uint256 => address) storage addrMap // Mapping trailing
            ) internal {
                // Process mapping
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("MappingTest", |c| {
            c.variable("balances", |v| {
                v.leading("// Simple mapping")
                    .ty("mapping(address => uint256)")
                    .visibility("public")
            })
            .variable("approvals", |v| {
                v.leading("// Nested mapping")
                    .ty("mapping(address => mapping(uint256 => bool))")
                    .visibility("public")
            })
            .struct_def("UserInfo", |s| {
                s.leading("// Complex mapping with struct")
                    .field("name", |f| f.ty("string"))
                    .field("age", |f| f.ty("uint256"))
            })
            .variable("users", |v| {
                v.leading("// Mapping to struct")
                    .ty("mapping(address => UserInfo)")
                    .visibility("public")
            })
            .function("processMapping", |f| {
                f.leading("// Mapping in function param")
                    .visibility("internal")
                    .parameter("addrMap", |p| {
                        p.ty("mapping(uint256 => address)")
                            .storage("storage")
                            .leading("// Mapping parameter")
                            .trailing("// Mapping trailing")
                    })
                    .body_statement(|s| s.nested_standalone_comment("// Process mapping"))
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_array_declarations_with_comments() {
        let source = r#"
        contract ArrayTest {
            // Dynamic array
            uint256[] public numbers;
            
            // Fixed size array
            address[10] public addresses;
            
            // 2D array
            uint256[][] public matrix;
            
            // Array of structs
            struct Item {
                string name;
                uint256 value;
            }
            
            // Struct array
            Item[] public items;
            
            // Function with array params
            function processArrays(
                // Dynamic array param
                uint256[] memory nums,
                // Fixed array param
                address[5] memory addrs
            ) public pure returns (
                // Array return
                uint256[] memory
            ) {
                return nums;
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("ArrayTest", |c| {
            c.variable("numbers", |v| {
                v.leading("// Dynamic array")
                    .ty("uint256[]")
                    .visibility("public")
            })
            .variable("addresses", |v| {
                v.leading("// Fixed size array")
                    .ty("address[10]")
                    .visibility("public")
            })
            .variable("matrix", |v| {
                v.leading("// 2D array")
                    .ty("uint256[][]")
                    .visibility("public")
            })
            .struct_def("Item", |s| {
                s.leading("// Array of structs")
                    .field("name", |f| f.ty("string"))
                    .field("value", |f| f.ty("uint256"))
            })
            .variable("items", |v| {
                v.leading("// Struct array")
                    .ty("Item[]")
                    .visibility("public")
            })
            .function("processArrays", |f| {
                f.leading("// Function with array params")
                    .visibility("public")
                    .mutability("pure")
                    .parameter("nums", |p| {
                        p.ty("uint256[]")
                            .storage("memory")
                            .leading("// Dynamic array param")
                    })
                    .parameter("addrs", |p| {
                        p.ty("address[5]")
                            .storage("memory")
                            .leading("// Fixed array param")
                    })
                    .returns_with_comment("uint256[]", "memory", "// Array return")
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_user_defined_value_types() {
        let source = r#"
        // User-defined value type for 18 decimal fixed point
        type UD60x18 is uint256;
        
        // Global using for the type
        using {
            add as +,
            sub as -,
            mul as *,
            div as /
        } for UD60x18 global;
        
        // Library functions
        function add(UD60x18 a, UD60x18 b) pure returns (UD60x18) {
            return UD60x18.wrap(UD60x18.unwrap(a) + UD60x18.unwrap(b));
        }
        
        function sub(UD60x18 a, UD60x18 b) pure returns (UD60x18) {
            return UD60x18.wrap(UD60x18.unwrap(a) - UD60x18.unwrap(b));
        }
        
        function mul(UD60x18 a, UD60x18 b) pure returns (UD60x18) {
            return UD60x18.wrap(UD60x18.unwrap(a) * UD60x18.unwrap(b) / 1e18);
        }
        
        function div(UD60x18 a, UD60x18 b) pure returns (UD60x18) {
            return UD60x18.wrap(UD60x18.unwrap(a) * 1e18 / UD60x18.unwrap(b));
        }
        
        contract UDVTUser {
            // Local type alias
            type Price is uint128;
            
            // Using type in storage
            UD60x18 public constant PI = UD60x18.wrap(3141592653589793238);
            
            // Function using UDVT
            function calculate(UD60x18 x, UD60x18 y) public pure returns (UD60x18) {
                // Operations use overloaded operators
                return (x + y) * PI / x;
            }
            
            // Function with local UDVT
            function priceExample() public pure {
                // Create price
                Price p1 = Price.wrap(100);
                Price p2 = Price.wrap(200);
                
                // Must unwrap to operate
                uint128 sum = Price.unwrap(p1) + Price.unwrap(p2);
                
                // Rewrap result
                Price total = Price.wrap(sum);
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new()
            .type_def("UD60x18", |t| {
                t.leading("// User-defined value type for 18 decimal fixed point")
            })
            .using(|u| {
                u.leading("// Global using for the type")
            })
            .function("add", |f| {
                f.leading("// Library functions")
                    .mutability("pure")
                    .returns("UD60x18")
                    .parameter("a", |p| p.ty("UD60x18"))
                    .parameter("b", |p| p.ty("UD60x18"))
                    .body_statement(|s| {
                        s.nested_statement(|s| s)
                    })
            })
            .function("sub", |f| {
                f.mutability("pure")
                    .returns("UD60x18")
                    .parameter("a", |p| p.ty("UD60x18"))
                    .parameter("b", |p| p.ty("UD60x18"))
                    .body_statement(|s| {
                        s.nested_statement(|s| s)
                    })
            })
            .function("mul", |f| {
                f.mutability("pure")
                    .returns("UD60x18")
                    .parameter("a", |p| p.ty("UD60x18"))
                    .parameter("b", |p| p.ty("UD60x18"))
                    .body_statement(|s| {
                        s.nested_statement(|s| s)
                    })
            })
            .function("div", |f| {
                f.mutability("pure")
                    .returns("UD60x18")
                    .parameter("a", |p| p.ty("UD60x18"))
                    .parameter("b", |p| p.ty("UD60x18"))
                    .body_statement(|s| {
                        s.nested_statement(|s| s)
                    })
            })
            .contract("UDVTUser", |c| {
                c.variable("PI", |v| {
                    v.leading("// Using type in storage")
                        .ty("UD60x18")
                        .visibility("public")
                        .constant()
                })
                .function("calculate", |f| {
                    f.leading("// Function using UDVT")
                        .visibility("public")
                        .mutability("pure")
                        .returns("UD60x18")
                        .parameter("x", |p| p.ty("UD60x18"))
                        .parameter("y", |p| p.ty("UD60x18"))
                        .body_statement(|s| {
                            s.nested_statement(|s| {
                                s.leading("// Operations use overloaded operators")
                            })
                        })
                })
                .function("priceExample", |f| {
                    f.leading("// Function with local UDVT")
                        .visibility("public")
                        .mutability("pure")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Create price"))
                                .nested_statement(|s| s)
                                .nested_statement(|s| s.leading("// Must unwrap to operate"))
                                .nested_statement(|s| s.leading("// Rewrap result"))
                        })
                })
            });

        expected.assert_matches(&collected);
    }
}