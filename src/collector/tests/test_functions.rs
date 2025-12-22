#[cfg(test)]
mod tests {
    use crate::collector::collect_source_unit;
    use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
    use solang_parser::parse;

    #[test]
    fn test_function_body_comments() {
        let source = r#"contract Test {
        function doSomething() public {
            // This comment is inside the function body
            uint x = 1;
            
            // Another comment
            x = x + 1;
        }
    }"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        // Use the builder pattern for cleaner assertions
        let expected = ExpectationBuilder::new().contract("Test", |c| {
            c.function("doSomething", |f| {
                f.visibility("public").body_statement(|s| {
                    s.nested_statement(|s| s.leading("// This comment is inside the function body"))
                        .nested_statement(|s| s.leading("// Another comment"))
                })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_function_attributes() {
        // Test that function visibility, mutability, and modifiers are properly verified
        let source = r#"contract FunctionTest {
        // Simple function with visibility
        function publicFunc() public {
            // Public function body
        }

        // External view function
        function getData() external view returns (uint) {
            return 42;
        }

        // Internal pure function
        function calculate(uint a, uint b) internal pure returns (uint) {
            return a + b;
        }

        // Private function
        function _helper() private {
            // Private helper
        }

        // Payable function
        function deposit() public payable {
            // Accept ether
        }

        // Virtual function
        function virtualFunc() public virtual {
            // Can be overridden
        }

        // Override function
        function someFunc() public override {
            // Overrides parent
        }

        // With modifiers
        modifier onlyOwner() {
            require(msg.sender == owner);
            _;
        }

        function restricted() public onlyOwner {
            // Restricted access
        }

        // Multiple modifiers
        function multiRestricted() public onlyOwner nonReentrant whenNotPaused {
            // Multiple restrictions
        }
    }"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("FunctionTest", |c| {
            c.function("publicFunc", |f| {
                f.visibility("public")
                    .leading("// Simple function with visibility")
                    .body_statement(|s| s.nested_standalone_comment("// Public function body"))
            })
            .function("getData", |f| {
                f.visibility("external")
                    .mutability("view")
                    .leading("// External view function")
                    .body_statement(|s| s.nested_statement(|s| s))
            })
            .function("calculate", |f| {
                f.visibility("internal")
                    .mutability("pure")
                    .parameter("a", |p| p.ty("uint256"))
                    .parameter("b", |p| p.ty("uint256"))
                    .leading("// Internal pure function")
                    .body_statement(|s| s.nested_statement(|s| s))
            })
            .function("_helper", |f| {
                f.visibility("private")
                    .leading("// Private function")
                    .body_statement(|s| s.nested_standalone_comment("// Private helper"))
            })
            .function("deposit", |f| {
                f.visibility("public")
                    .mutability("payable")
                    .leading("// Payable function")
                    .body_statement(|s| s.nested_standalone_comment("// Accept ether"))
            })
            .function("virtualFunc", |f| {
                f.visibility("public")
                    .is_virtual()
                    .leading("// Virtual function")
                    .body_statement(|s| s.nested_standalone_comment("// Can be overridden"))
            })
            .function("someFunc", |f| {
                f.visibility("public")
                    .is_override()
                    .leading("// Override function")
                    .body_statement(|s| s.nested_standalone_comment("// Overrides parent"))
            })
            .function("onlyOwner", |f| {
                f.leading("// With modifiers")
                    .body_statement(|s| s.nested_statement(|s| s).nested_statement(|s| s))
            })
            .function("restricted", |f| {
                f.visibility("public")
                    .modifier("onlyOwner")
                    .body_statement(|s| s.nested_standalone_comment("// Restricted access"))
            })
            .function("multiRestricted", |f| {
                f.visibility("public")
                    .modifier("onlyOwner")
                    .modifier("nonReentrant")
                    .modifier("whenNotPaused")
                    .leading("// Multiple modifiers")
                    .body_statement(|s| s.nested_standalone_comment("// Multiple restrictions"))
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_function_parameters() {
        // Test that function parameters with types and storage locations are properly verified
        let source = r#"contract ParameterTest {
        // Simple parameters
        function transfer(address to, uint256 amount) public {
            // Transfer logic
        }

        // Array parameters
        function batchTransfer(address[] memory recipients, uint256[] memory amounts) public {
            // Batch transfer logic
        }

        // Struct parameters  
        struct Data {
            uint256 value;
            string name;
        }

        // Complex parameter types
        function updateData(Data memory data, bytes calldata extraData) external {
            // Update logic
        }

        // Complex types with arrays of structs
        function updateMapping(address[] storage addresses, ValueStruct[] memory values) internal {
            // Complex parameter types
            // Update mapping
        }
    }"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("ParameterTest", |c| {
            c.function("transfer", |f| {
                f.visibility("public")
                    .parameter("to", |p| p.ty("address"))
                    .parameter("amount", |p| p.ty("uint256"))
                    .leading("// Simple parameters")
                    .body_statement(|s| s.nested_standalone_comment("// Transfer logic"))
            })
            .function("batchTransfer", |f| {
                f.visibility("public")
                    .parameter("recipients", |p| {
                        p.ty("address[]").storage_location("memory")
                    })
                    .parameter("amounts", |p| p.ty("uint256[]").storage_location("memory"))
                    .leading("// Array parameters")
                    .body_statement(|s| s.nested_standalone_comment("// Batch transfer logic"))
            })
            .struct_def("Data", |s| {
                s.leading("// Struct parameters  ")
                    .field("value", |f| f.ty("uint256"))
                    .field("name", |f| f.ty("string"))
            })
            .function("updateData", |f| {
                f.visibility("external")
                    .parameter("data", |p| p.ty("Data").storage_location("memory"))
                    .parameter("extraData", |p| p.ty("bytes").storage_location("calldata"))
                    .leading("// Complex parameter types")
                    .body_statement(|s| s.nested_standalone_comment("// Update logic"))
            })
            .function("updateMapping", |f| {
                f.visibility("internal")
                    .parameter("addresses", |p| {
                        p.ty("address[]").storage_location("storage")
                    })
                    .parameter("values", |p| {
                        p.ty("ValueStruct[]").storage_location("memory")
                    })
                    .leading("// Complex types with arrays of structs")
                    .body_statement(|s| {
                        s.nested_standalone_comment("// Complex parameter types")
                            .nested_standalone_comment("// Update mapping")
                    })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_function_parameter_and_return_comments() {
        let source = r#"
        contract ParameterCommentTest {
            function transfer(
                // The recipient address
                address to,
                // Amount to transfer
                uint256 amount
            ) public returns (
                // Success status
                bool
            ) {
                return true;
            }
            
            function multiReturn() public returns (
                // First value
                uint256,
                // Second value
                address,
                // Third value
                bool success
            ) {
                return (42, address(0), true);
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("ParameterCommentTest", |c| {
            c.function("transfer", |f| {
                f.visibility("public")
                    .parameter("to", |p| {
                        p.ty("address").leading("// The recipient address")
                    })
                    .parameter("amount", |p| {
                        p.ty("uint256").leading("// Amount to transfer")
                    })
                    .return_param(|r| r.ty("bool").leading("// Success status"))
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
            .function("multiReturn", |f| {
                f.visibility("public")
                    .return_param(|r| r.ty("uint256").leading("// First value"))
                    .return_param(|r| r.ty("address").leading("// Second value"))
                    .return_param(|r| r.ty("bool").leading("// Third value"))
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_multiple_return_values_with_comments() {
        let source = r#"
        contract MultiReturnTest {
            // Function with named returns
            function getValues() public pure returns (
                // First return value
                uint256 amount,
                // Second return value  
                address recipient,
                // Success flag
                bool success
            ) {
                amount = 100;
                recipient = address(0);
                success = true;
            }
            
            // Function with unnamed returns
            function getUnnamedValues() public pure returns (
                // Amount value
                uint256,
                // Address value
                address,
                // Status flag
                bool
            ) {
                return (200, address(0), false);
            }
            
            // Mixed named and unnamed (edge case)
            function getMixedValues() public pure returns (
                uint256 value, // Named value
                // Unnamed address
                address
            ) {
                value = 300;
                return (value, address(0));
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("MultiReturnTest", |c| {
            c.function("getValues", |f| {
                f.leading("// Function with named returns")
                    .visibility("public")
                    .mutability("pure")
                    .return_param(|r| r.ty("uint256").leading("// First return value"))
                    .return_param(|r| r.ty("address").leading("// Second return value  "))
                    .return_param(|r| r.ty("bool").leading("// Success flag"))
                    .body_statement(|s| {
                        s.nested_statement(|s| s) // amount = 100
                            .nested_statement(|s| s) // recipient = address(0)
                            .nested_statement(|s| s) // success = true
                    })
            })
            .function("getUnnamedValues", |f| {
                f.leading("// Function with unnamed returns")
                    .visibility("public")
                    .mutability("pure")
                    .return_param(|r| r.ty("uint256").leading("// Amount value"))
                    .return_param(|r| r.ty("address").leading("// Address value"))
                    .return_param(|r| r.ty("bool").leading("// Status flag"))
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
            .function("getMixedValues", |f| {
                f.leading("// Mixed named and unnamed (edge case)")
                    .visibility("public")
                    .mutability("pure")
                    .return_param(|r| r.ty("uint256").trailing("// Named value"))
                    .return_param(|r| r.ty("address").leading("// Unnamed address"))
                    .body_statement(|s| {
                        s.nested_statement(|s| s) // value = 300
                            .nested_statement(|s| s) // return statement
                    })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_functions_with_unnamed_parameters() {
        let source = r#"
        contract UnnamedParameterTest {
            // Interface function with unnamed params
            function transfer(
                // Recipient address comment
                address,
                // Transfer amount comment
                uint256
            ) external returns (bool);
            
            // Function with mix of named and unnamed
            function processData(
                // Named parameter
                bytes32 id,
                // Unnamed data parameter
                bytes memory,
                // Another unnamed param
                uint256
            ) public pure {
                // Process the data
            }
            
            // Abstract function with all unnamed
            function abstractMethod(
                // First unnamed param
                uint256,
                // Second unnamed param
                address,
                // Third unnamed param with trailing
                bool // Important flag
            ) public virtual;
            
            // Function type parameter with unnamed params
            function executeCallback(
                // Callback function with unnamed params
                function(uint256, address) external returns (bool) callback,
                // Data for callback
                uint256 data
            ) public {
                callback(data, msg.sender);
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("UnnamedParameterTest", |c| {
            c.function("transfer", |f| {
                f.leading("// Interface function with unnamed params")
                    .visibility("external")
                    .returns("bool")
                    .parameter("", |p| {
                        p.ty("address")
                            .leading("// Recipient address comment")
                    })
                    .parameter("", |p| {
                        p.ty("uint256")
                            .leading("// Transfer amount comment")
                    })
                    // Interface function - no body
            })
            .function("processData", |f| {
                f.leading("// Function with mix of named and unnamed")
                    .visibility("public")
                    .mutability("pure")
                    .parameter("id", |p| {
                        p.ty("bytes32")
                            .leading("// Named parameter")
                    })
                    .parameter("", |p| {
                        p.ty("bytes")
                            .storage("memory")
                            .leading("// Unnamed data parameter")
                    })
                    .parameter("", |p| {
                        p.ty("uint256")
                            .leading("// Another unnamed param")
                    })
                    .body_statement(|s| s.nested_standalone_comment("// Process the data"))
            })
            .function("abstractMethod", |f| {
                f.leading("// Abstract function with all unnamed")
                    .visibility("public")
                    .is_virtual()
                    .parameter("", |p| {
                        p.ty("uint256")
                            .leading("// First unnamed param")
                    })
                    .parameter("", |p| {
                        p.ty("address")
                            .leading("// Second unnamed param")
                    })
                    .parameter("", |p| {
                        p.ty("bool")
                            .leading("// Third unnamed param with trailing")
                            .trailing("// Important flag")
                    })
                    // Abstract function - no body
            })
            .function("executeCallback", |f| {
                f.leading("// Function type parameter with unnamed params")
                    .visibility("public")
                    .parameter("callback", |p| {
                        p.leading("// Callback function with unnamed params")
                            // Don't check function type strings as they're complex
                    })
                    .parameter("data", |p| {
                        p.ty("uint256")
                            .leading("// Data for callback")
                    })
                    .body_statement(|s| s.nested_statement(|s| s)) // callback call
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_receive_and_fallback_functions() {
        let source = r#"
        // Contract with special functions
        contract PaymentReceiver {
            // Event for logging
            event PaymentReceived(address from, uint256 amount);
            
            // Receive function for plain ETH transfers
            receive() external payable {
                // Log the payment
                emit PaymentReceived(msg.sender, msg.value);
            }
            
            // Fallback for unknown function calls
            fallback() external payable {
                // Handle unknown calls
                if (msg.data.length > 0) {
                    // Call had data
                    revert("Unknown function");
                }
                // No data - treat as payment
            }
        }
        
        // Contract with no-data fallback
        contract SimpleFallback {
            // Fallback without payable
            fallback() external {
                // Cannot receive ETH
                revert(); // Simple revert
            }
        }
        
        // Abstract contract with virtual fallback
        abstract contract Upgradeable {
            // Virtual fallback for upgrades
            fallback() external payable virtual {
                // Delegate to implementation
                _delegate(implementation());
            }
            
            // Abstract function
            function implementation() internal view virtual returns (address);
            
            // Internal delegation function
            function _delegate(address impl) internal {
                // Delegate call implementation
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new()
            .contract("PaymentReceiver", |c| {
                c.leading("// Contract with special functions")
                    .event("PaymentReceived", |e| {
                        e.leading("// Event for logging")
                            .parameter("from", |p| p.ty("address"))
                            .parameter("amount", |p| p.ty("uint256"))
                    })
                    .function("", |f| {
                        f.leading("// Receive function for plain ETH transfers")
                            .visibility("external")
                            .mutability("payable")
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Log the payment"))
                            })
                    })
                    .function("", |f| {
                        f.leading("// Fallback for unknown function calls")
                            .visibility("external")
                            .mutability("payable")
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Handle unknown calls")
                                        .nested_statement(|s| {
                                            s.leading("// Call had data")
                                        })
                                })
                                .nested_standalone_comment("// No data - treat as payment")
                            })
                    })
            })
            .contract("SimpleFallback", |c| {
                c.leading("// Contract with no-data fallback")
                    .function("", |f| {
                        f.leading("// Fallback without payable")
                            .visibility("external")
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Cannot receive ETH")
                                        .trailing("// Simple revert")
                                })
                            })
                    })
            })
            .contract("Upgradeable", |c| {
                c.leading("// Abstract contract with virtual fallback")
                    .contract_type("abstract")
                    .function("", |f| {
                        f.leading("// Virtual fallback for upgrades")
                            .visibility("external")
                            .mutability("payable")
                            .is_virtual()
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Delegate to implementation"))
                            })
                    })
                    .function("implementation", |f| {
                        f.leading("// Abstract function")
                            .visibility("internal")
                            .mutability("view")
                            .is_virtual()
                            .returns("address")
                    })
                    .function("_delegate", |f| {
                        f.leading("// Internal delegation function")
                            .visibility("internal")
                            .parameter("impl", |p| p.ty("address"))
                            .body_statement(|s| {
                                s.nested_standalone_comment("// Delegate call implementation")
                            })
                    })
            });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_function_types_with_comments() {
        let source = r#"
        contract FunctionTypeTest {
            // Function type state variable
            function(uint256, uint256) external pure returns (uint256) mathOp;
            
            // Function accepting function type
            function executeOperation(
                // The operation function
                function(uint256, uint256) external pure returns (uint256) op,
                // First operand
                uint256 a,
                // Second operand
                uint256 b
            ) public pure returns (
                // Result of operation
                uint256
            ) {
                return op(a, b);
            }
            
            // Function returning function type
            function getOperation(
                // Operation selector
                uint8 opType
            ) public pure returns (
                // Selected operation function
                function(uint256, uint256) external pure returns (uint256)
            ) {
                return add;
            }
            
            // Add function
            function add(uint256 x, uint256 y) public pure returns (uint256) {
                return x + y;
            }
            
            // Multiply function
            function multiply(uint256 x, uint256 y) public pure returns (uint256) {
                return x * y;
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("FunctionTypeTest", |c| {
            c.variable("mathOp", |v| {
                v.leading("// Function type state variable")
                    // Don't check the exact type string for function types as it's complex
            })
            .function("executeOperation", |f| {
                f.leading("// Function accepting function type")
                    .visibility("public")
                    .mutability("pure")
                    .parameter("op", |p| {
                        p.leading("// The operation function")
                            // Don't check function type strings
                    })
                    .parameter("a", |p| {
                        p.ty("uint256").leading("// First operand")
                    })
                    .parameter("b", |p| {
                        p.ty("uint256").leading("// Second operand")
                    })
                    .return_param(|r| r.ty("uint256").leading("// Result of operation"))
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
            .function("getOperation", |f| {
                f.leading("// Function returning function type")
                    .visibility("public")
                    .mutability("pure")
                    .parameter("opType", |p| {
                        p.ty("uint8").leading("// Operation selector")
                    })
                    .return_param(|r| {
                        r.leading("// Selected operation function")
                            // Don't check function type strings
                    })
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
            .function("add", |f| {
                f.leading("// Add function")
                    .visibility("public")
                    .mutability("pure")
                    .parameter("x", |p| p.ty("uint256"))
                    .parameter("y", |p| p.ty("uint256"))
                    .returns("uint256")
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
            .function("multiply", |f| {
                f.leading("// Multiply function")
                    .visibility("public")
                    .mutability("pure")
                    .parameter("x", |p| p.ty("uint256"))
                    .parameter("y", |p| p.ty("uint256"))
                    .returns("uint256")
                    .body_statement(|s| s.nested_statement(|s| s)) // return statement
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_multiline_function_calls_with_named_args() {
        let source = r#"
        interface IComplexSystem {
            function process(
                address user,
                uint256 amount,
                bytes data,
                bool urgent
            ) external returns (bool);
        }
        
        contract NamedArgumentCalls {
            // Complex function with many parameters
            function complexOperation(
                address target,
                uint256 value,
                bytes memory data,
                uint256 gasLimit,
                address refundTo,
                bool requireSuccess
            ) public returns (bytes memory) {
                // Implementation
                return data;
            }
            
            // Function using named arguments
            function callWithNamedArgs() public {
                // Single-line named arguments
                complexOperation({
                    target: 0x1234567890123456789012345678901234567890,
                    value: 1 ether,
                    data: hex"abcdef",
                    gasLimit: 100000,
                    refundTo: msg.sender,
                    requireSuccess: true
                });
                
                // Multi-line named arguments with comments
                complexOperation({
                    target: msg.sender,     // Use caller as target
                    value: 0,              // No ETH transfer
                    data: abi.encode(      // Encode parameters
                        "hello",
                        42
                    ),
                    gasLimit: 50000,       // Conservative gas limit
                    refundTo: address(0),  // Burn any refund
                    requireSuccess: false  // Allow failure
                });
                
                // Mixed positional and named (if supported)
                IComplexSystem(msg.sender).process{
                    value: 1 ether,    // Send ETH
                    gas: 200000       // Gas override
                }(
                    msg.sender,        // user param
                    100,              // amount param
                    "",               // empty data
                    true              // urgent flag
                );
            }
            
            // Nested function calls with named args
            function nestedCalls() public {
                // Outer call with named args
                complexOperation({
                    target: address(this),
                    value: 0,
                    data: abi.encodeCall(
                        // Inner call reference
                        this.complexOperation,
                        (
                            msg.sender,  // Nested target
                            1 ether,     // Nested value
                            "",          // Nested data
                            30000,       // Nested gas
                            msg.sender,  // Nested refund
                            true         // Nested require
                        )
                    ),
                    gasLimit: 100000,
                    refundTo: msg.sender,
                    requireSuccess: true
                });
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new()
            .contract("IComplexSystem", |c| {
                c.contract_type("interface")
                    .function("process", |f| {
                        f.visibility("external")
                            .returns("bool")
                            .parameter("user", |p| p.ty("address"))
                            .parameter("amount", |p| p.ty("uint256"))
                            .parameter("data", |p| p.ty("bytes"))
                            .parameter("urgent", |p| p.ty("bool"))
                    })
            })
            .contract("NamedArgumentCalls", |c| {
                c.function("complexOperation", |f| {
                    f.leading("// Complex function with many parameters")
                        .visibility("public")
                        .returns("bytes")
                        .return_param(|r| r.storage("memory"))
                        .parameter("target", |p| p.ty("address"))
                        .parameter("value", |p| p.ty("uint256"))
                        .parameter("data", |p| p.ty("bytes").storage("memory"))
                        .parameter("gasLimit", |p| p.ty("uint256"))
                        .parameter("refundTo", |p| p.ty("address"))
                        .parameter("requireSuccess", |p| p.ty("bool"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Implementation"))
                        })
                })
                .function("callWithNamedArgs", |f| {
                    f.leading("// Function using named arguments")
                        .visibility("public")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Single-line named arguments"))
                                .nested_statement(|s| s.leading("// Multi-line named arguments with comments"))
                                .nested_statement(|s| s.leading("// Mixed positional and named (if supported)"))
                        })
                })
                .function("nestedCalls", |f| {
                    f.leading("// Nested function calls with named args")
                        .visibility("public")
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Outer call with named args"))
                        })
                })
            });

        expected.assert_matches(&collected);
    }
}