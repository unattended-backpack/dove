#[cfg(test)]
mod tests {
    use crate::collector::collect_source_unit;
    use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
    use solang_parser::parse;

    #[test]
    fn test_complex_type_expressions_with_named_mappings() {
        // This test demonstrates the current behavior: comments within type expressions
        // are NOT collected. This test establishes the baseline for the feature we want to add.
        let source = r#"
        contract ComplexMappings {
            // Simple mapping with named keys and values
            mapping(
                // User address
                address user => // Maps to balance
                uint256 balance // User's token balance
            ) public balances;
            
            // Nested mapping with all named
            mapping(
                // Token contract address
                address token => // Maps to owner mapping
                mapping(
                    // Token owner address  
                    address owner => // Maps to spender mapping
                    mapping(
                        // Approved spender
                        address spender => // Maps to allowance
                        uint256 amount // Approved amount
                    ) allowance // Nested allowance mapping
                ) owners // Token owners mapping
            ) public tokenAllowances;
            
            // Triple nested mapping
            mapping(
                // Protocol identifier
                uint256 protocolId => // Maps to version
                mapping(
                    // Version number
                    uint8 version => // Maps to contract
                    mapping(
                        // Contract type
                        bytes32 contractType => // Maps to implementation
                        address implementation // Implementation address
                    ) contracts // Contracts by type
                ) versions // Versions mapping
            ) public protocols;
            
            // Mapping in struct
            struct UserData {
                // User's name
                string name;
                // User's roles mapping
                mapping(
                    // Role identifier
                    bytes32 role => // Has role?
                    bool hasRole // Role assignment
                ) roles;
            }
            
            // Array of mappings
            mapping(
                // Category ID
                uint256 category => // Maps to items
                uint256[] items // Item IDs in category
            )[] public categorizedItems;
            
            // Function with complex mapping parameter
            function complexParameter(
                // Complex mapping parameter
                mapping(
                    // Key type
                    address key => // Maps to value
                    uint256 value // The value
                ) storage myMap // Storage reference
            ) internal {
                // Function body
            }
            
            // Complex types in function signature
            function multipleComplexParams(
                mapping(
                    address /* user */ => // User balance
                    uint256 /* balance */
                ) storage balances,
                mapping(
                    address => // Primary key
                    mapping(
                        address => // Secondary key
                        uint256 // Allowance value
                    )
                ) storage allowances,
                uint256[] memory amounts
            ) internal view returns (
                mapping(
                    uint256 => // Input value
                    uint256 // Output value
                ) storage result
            ) {
                // Implementation
            }
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("ComplexMappings", |c| {
                c.variable("balances", |v| {
                    v.leading("// Simple mapping with named keys and values")
                        .ty("mapping(address => uint256)")
                        .visibility("public")
                        .type_comments(|t| {
                            t.key_leading("// User address")
                             .key_trailing("// Maps to balance")
                             .value_trailing("// User's token balance")
                        })
                })
                .variable("tokenAllowances", |v| {
                    v.leading("// Nested mapping with all named")
                        .ty("mapping(address => mapping(address => mapping(address => uint256)))")
                        .visibility("public")
                        // TODO: Add nested type comment expectations
                        // .type_comments(|t| {
                        //     t.key_leading("// Token contract address")
                        //      .key_trailing("// Maps to owner mapping")
                        //      .value_trailing("// Token owners mapping")
                        //      .value_type_comments(|t2| {
                        //          t2.key_leading("// Token owner address")
                        //            .key_trailing("// Maps to spender mapping")
                        //            .value_type_comments(|t3| {
                        //                t3.key_leading("// Approved spender")
                        //                  .key_trailing("// Maps to allowance")
                        //                  .value_trailing("// Approved amount")
                        //            })
                        //            .value_trailing("// Nested allowance mapping")
                        //      })
                        // })
                })
                .variable("protocols", |v| {
                    v.leading("// Triple nested mapping")
                        .ty("mapping(uint256 => mapping(uint8 => mapping(bytes32 => address)))")
                        .visibility("public")
                        // TODO: Add triple nested type comments
                })
                .struct_def("UserData", |s| {
                    s.leading("// Mapping in struct")
                        .field("name", |f| f.leading("// User's name").ty("string"))
                        .field("roles", |f| {
                            f.leading("// User's roles mapping")
                                .ty("mapping(bytes32 => bool)")
                                // TODO: Add type comments for struct field
                                // .type_comments(|t| {
                                //     t.key_leading("// Role identifier")
                                //      .key_trailing("// Has role?")
                                //      .value_trailing("// Role assignment")
                                // })
                        })
                })
                .variable("categorizedItems", |v| {
                    v.leading("// Array of mappings")
                        .ty("mapping(uint256 => uint256[])[]")
                        .visibility("public")
                        // TODO: Add type comments for array of mappings
                })
                .function("complexParameter", |f| {
                    f.leading("// Function with complex mapping parameter")
                        .visibility("internal")
                        .parameter("myMap", |p| {
                            p.leading("// Complex mapping parameter")
                                .ty("mapping(address => uint256)")
                                .storage("storage")
                                .trailing("// Storage reference")
                                // TODO: Add type comments for parameter
                                // .type_comments(|t| {
                                //     t.key_leading("// Key type")
                                //      .key_trailing("// Maps to value")
                                //      .value_trailing("// The value")
                                // })
                        })
                        .body_statement(|s| {
                            s.nested_standalone_comment("// Function body")
                        })
                })
                .function("multipleComplexParams", |f| {
                    f.leading("// Complex types in function signature")
                        .visibility("internal")
                        .mutability("view")
                        .parameter("balances", |p| {
                            p.ty("mapping(address => uint256)")
                                .storage("storage")
                        })
                        .parameter("allowances", |p| {
                            p.ty("mapping(address => mapping(address => uint256))")
                                .storage("storage")
                        })
                        .parameter("amounts", |p| {
                            p.ty("uint256[]")
                                .storage("memory")
                        })
                        .returns("mapping(uint256 => uint256)")
                        .return_param(|r| r.storage("storage"))
                        .body_statement(|s| {
                            s.nested_standalone_comment("// Implementation")
                        })
                })
            });
        expected.assert_matches(&collected);
    }

    #[test]
    fn test_array_and_struct_literals_with_comments() {
        let source = r#"
        contract LiteralsWithComments {
            // Struct definitions
            struct Point {
                uint256 x;
                uint256 y;
            }
            
            struct Person {
                string name;
                uint256 age;
                address wallet;
                bool active;
            }
            
            // Array literal with comments
            function createArray() public pure returns (uint256[] memory) {
                // Simple array
                uint256[] memory simple = [1, 2, 3];
                
                // Array with inline comments
                uint256[] memory numbers = [
                    100,    // First hundred
                    200,    // Second hundred
                    300,    // Third hundred
                    400     // Fourth hundred
                ];
                
                // Nested arrays
                uint256[][] memory matrix = [
                    [1, 2, 3],      // First row
                    [4, 5, 6],      // Second row
                    [7, 8, 9]       // Third row
                ];
                
                return numbers;
            }
            
            // Struct literal with comments
            function createStructs() public view returns (Person memory) {
                // Simple struct
                Point memory origin = Point(0, 0);
                
                // Struct with named fields and comments
                Point memory point = Point({
                    x: 100,     // X coordinate
                    y: 200      // Y coordinate
                });
                
                // Complex struct with comments
                Person memory person = Person({
                    name: "Alice",           // User's name
                    age: 30,                // User's age
                    wallet: msg.sender,     // User's wallet address
                    active: true            // Account status
                });
                
                // Nested struct creation
                Person memory admin = Person({
                    name: "Admin",
                    age: 0,                 // Age not tracked for admin
                    wallet: address(this),  // Contract is the admin
                    active: true
                });
                
                return person;
            }
            
            // Struct with array field
            struct Group {
                string name;
                uint256[] memberIds;
            }
            
            // Mixed literals
            function mixedLiterals() public pure {
                // Array of structs
                Point[] memory points = [
                    Point(0, 0),      // Origin
                    Point(10, 0),     // X-axis
                    Point(0, 10),     // Y-axis
                    Point(10, 10)     // Diagonal
                ];
                
                // Create group struct
                Group memory admins = Group({
                    name: "Administrators",     // Group name
                    memberIds: [1, 2, 3, 4]    // Admin IDs
                });
            }
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("LiteralsWithComments", |c| {
                c.struct_def("Point", |s| {
                    s.leading("// Struct definitions")
                        .field("x", |f| f.ty("uint256"))
                        .field("y", |f| f.ty("uint256"))
                })
                .struct_def("Person", |s| {
                    s.field("name", |f| f.ty("string"))
                        .field("age", |f| f.ty("uint256"))
                        .field("wallet", |f| f.ty("address"))
                        .field("active", |f| f.ty("bool"))
                })
                .struct_def("Group", |s| {
                    s.leading("// Struct with array field")
                        .field("name", |f| f.ty("string"))
                        .field("memberIds", |f| f.ty("uint256[]"))
                })
                .function("createArray", |f| {
                    f.leading("// Array literal with comments")
                        .visibility("public")
                        .mutability("pure")
                        .returns("uint256[]")
                        .return_param(|r| r.storage("memory"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Simple array"))
                                .nested_statement(|s| s.leading("// Array with inline comments"))
                                .nested_statement(|s| s.leading("// Nested arrays"))
                                .nested_statement(|s| s)
                        })
                })
                .function("createStructs", |f| {
                    f.leading("// Struct literal with comments")
                        .visibility("public")
                        .mutability("view")
                        .returns("Person")
                        .return_param(|r| r.storage("memory"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Simple struct"))
                                .nested_statement(|s| s.leading("// Struct with named fields and comments"))
                                .nested_statement(|s| s.leading("// Complex struct with comments"))
                                .nested_statement(|s| s.leading("// Nested struct creation"))
                                .nested_statement(|s| s)
                        })
                })
                .function("mixedLiterals", |f| {
                    f.leading("// Mixed literals")
                        .visibility("public")
                        .mutability("pure")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Array of structs"))
                                .nested_statement(|s| s.leading("// Create group struct"))
                        })
                })
            });
        expected.assert_matches(&collected);
    }

    #[test]
    fn test_tuple_assignments_and_declarations() {
        let source = r#"
        contract TupleOperations {
            // State variables for testing
            uint256 public x;
            uint256 public y;
            address public owner;
            bool public active;
            
            // Function returning multiple values
            function getMultiple() public view returns (uint256, address, bool) {
                return (42, msg.sender, true);
            }
            
            // Tuple declarations
            function tupleDeclarations() public {
                // Simple tuple declaration
                (uint256 a, uint256 b) = (1, 2);
                
                // Tuple with type inference
                var (c, d) = (3, 4);
                
                // Mixed types in tuple
                (uint256 amount, address recipient, bool success) = (
                    100 ether,      // Amount to send
                    msg.sender,     // Recipient address
                    true            // Success flag
                );
                
                // Nested tuples
                ((uint256 x1, uint256 y1), (uint256 x2, uint256 y2)) = (
                    (10, 20),       // First point
                    (30, 40)        // Second point
                );
            }
            
            // Tuple assignments
            function tupleAssignments() public {
                // Assign to existing variables
                (x, y) = (100, 200);
                
                // Partial assignments with gaps
                (, uint256 value, ) = getMultiple();
                
                // All gaps
                (,,) = getMultiple(); // Ignore all returns
                
                // Mixed declaration and assignment
                (uint256 newX, ) = (999, 0);
                
                // Assign to state variables
                (x, owner, active) = getMultiple();
                
                // Complex assignment with function call
                (x, owner, active) = (
                    block.timestamp,    // Current time as x
                    tx.origin,         // Transaction origin as owner  
                    block.number > 0   // Always true for active
                );
            }
            
            // Tuple returns and destructuring
            function complexTuples() public returns (uint256, uint256) {
                // Return tuple directly
                return (x, y);
                
                // Can't test more complex scenarios without actual execution
            }
            
            // Edge cases
            function tupleEdgeCases() public {
                // Single element tuple (not really a tuple in Solidity)
                (uint256 single) = (42);
                
                // Empty tuple for function with no returns
                () = noReturnFunction();
                
                // Tuple in conditional
                bool condition = true;
                (uint256 a, uint256 b) = condition 
                    ? (1, 2)    // True branch
                    : (3, 4);   // False branch
            }
            
            // Helper function
            function noReturnFunction() internal {
                // Does nothing
            }
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("TupleOperations", |c| {
                c.variable("x", |v| {
                    v.leading("// State variables for testing")
                        .ty("uint256")
                        .visibility("public")
                })
                .variable("y", |v| {
                    v.ty("uint256")
                        .visibility("public")
                })
                .variable("owner", |v| {
                    v.ty("address")
                        .visibility("public")
                })
                .variable("active", |v| {
                    v.ty("bool")
                        .visibility("public")
                })
                .function("getMultiple", |f| {
                    f.leading("// Function returning multiple values")
                        .visibility("public")
                        .mutability("view")
                        .returns("uint256")
                        .returns("address")
                        .returns("bool")
                        .body_statement(|s| {
                            s.nested_statement(|s| s)
                        })
                })
                .function("tupleDeclarations", |f| {
                    f.leading("// Tuple declarations")
                        .visibility("public")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Simple tuple declaration"))
                                .nested_statement(|s| s.leading("// Tuple with type inference"))
                                .nested_statement(|s| s.leading("// Mixed types in tuple"))
                                .nested_statement(|s| s.leading("// Nested tuples"))
                        })
                })
                .function("tupleAssignments", |f| {
                    f.leading("// Tuple assignments")
                        .visibility("public")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Assign to existing variables"))
                                .nested_statement(|s| s.leading("// Partial assignments with gaps"))
                                .nested_statement(|s| s.leading("// All gaps").trailing("// Ignore all returns"))
                                .nested_statement(|s| s.leading("// Mixed declaration and assignment"))
                                .nested_statement(|s| s.leading("// Assign to state variables"))
                                .nested_statement(|s| s.leading("// Complex assignment with function call"))
                        })
                })
                .function("complexTuples", |f| {
                    f.leading("// Tuple returns and destructuring")
                        .visibility("public")
                        .returns("uint256")
                        .returns("uint256")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Return tuple directly"))
                                .nested_standalone_comment("// Can't test more complex scenarios without actual execution")
                        })
                })
                .function("tupleEdgeCases", |f| {
                    f.leading("// Edge cases")
                        .visibility("public")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Single element tuple (not really a tuple in Solidity)"))
                                .nested_statement(|s| s.leading("// Empty tuple for function with no returns"))
                                .nested_statement(|s| s.leading("// Tuple in conditional"))  // bool condition = true;
                                .nested_statement(|s| s.trailing("// False branch"))
                        })
                })
                .function("noReturnFunction", |f| {
                    f.leading("// Helper function")
                        .visibility("internal")
                        .body_statement(|s| {
                            s.nested_standalone_comment("// Does nothing")
                        })
                })
            });
        expected.assert_matches(&collected);
    }

    #[test]
    fn test_complex_expressions_with_comments() {
        let source = r#"
        contract ComplexExpressions {
            // Test ternary with comments
            function ternaryWithComments(uint256 x) public pure returns (uint256) {
                return x > 10 
                    ? x * 2  // Double if large
                    : x + 1; // Increment if small
            }
            
            // Array literals with comments
            function arrayLiterals() public pure returns (uint256[] memory) {
                uint256[] memory arr = [
                    1,   // First element
                    2,   // Second element
                    3    // Third element
                ];
                return arr;
            }
            
            // Struct construction with comments
            struct Data {
                uint256 id;
                address owner;
                bool active;
            }
            
            function createStruct() public view returns (Data memory) {
                return Data({
                    id: 42,           // Unique identifier
                    owner: msg.sender, // Current caller
                    active: true      // Start active
                });
            }
            
            // Multi-line function calls
            function complexCall(address target) public {
                // Call with many arguments
                IComplex(target).doSomething{
                    value: 1 ether,  // Payment amount
                    gas: 100000     // Gas limit
                }(
                    msg.sender,      // First param
                    block.timestamp, // Second param
                    "data"          // Third param
                );
            }
            
            // Nested expressions with comments
            function nestedMath(uint256 a, uint256 b, uint256 c) public pure returns (uint256) {
                return (
                    a + // First value
                    b   // Second value
                ) * (
                    c - 1 // Subtract one from c
                );
            }
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("ComplexExpressions", |c| {
                c.function("ternaryWithComments", |f| {
                    f.leading("// Test ternary with comments")
                        .visibility("public")
                        .mutability("pure")
                        .returns("uint256")
                        .parameter("x", |p| p.ty("uint256"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s.trailing("// Increment if small"))
                        })
                })
                .function("arrayLiterals", |f| {
                    f.leading("// Array literals with comments")
                        .visibility("public")
                        .mutability("pure")
                        .returns("uint256[]")
                        .return_param(|r| r.storage("memory"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s)
                                .nested_statement(|s| s)
                        })
                })
                .struct_def("Data", |s| {
                    s.leading("// Struct construction with comments")
                        .field("id", |f| f.ty("uint256"))
                        .field("owner", |f| f.ty("address"))
                        .field("active", |f| f.ty("bool"))
                })
                .function("createStruct", |f| {
                    f.visibility("public")
                        .mutability("view")
                        .returns("Data")
                        .return_param(|r| r.storage("memory"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s)
                        })
                })
                .function("complexCall", |f| {
                    f.leading("// Multi-line function calls")
                        .visibility("public")
                        .parameter("target", |p| p.ty("address"))
                        .body_statement(|s| {
                            s.nested_statement(|s| {
                                s.leading("// Call with many arguments")
                            })
                        })
                })
                .function("nestedMath", |f| {
                    f.leading("// Nested expressions with comments")
                        .visibility("public")
                        .mutability("pure")
                        .returns("uint256")
                        .parameter("a", |p| p.ty("uint256"))
                        .parameter("b", |p| p.ty("uint256"))
                        .parameter("c", |p| p.ty("uint256"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s)
                        })
                })
            });
        expected.assert_matches(&collected);
    }
}