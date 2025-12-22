#[cfg(test)]
mod tests {
    use crate::collector::collect_source_unit;
    use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
    use solang_parser::parse;

    #[test]
    fn test_nested_statement_comments() {
        let source = r#"contract Test {
    function complexLogic() public {
        // Check if value is positive
        if (msg.value > 0) {
            // Process payment
            uint amount = msg.value;
            
            // Double the amount
            amount = amount * 2;
            
            // Send it back
            payable(msg.sender).transfer(amount);
        } else {
            // No payment received
            revert("No payment");
        }
    }
}"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        // Use the builder pattern for cleaner assertions
        let expected = ExpectationBuilder::new().contract("Test", |c| {
            c.function("complexLogic", |f| {
                f.visibility("public").body_statement(|s| {
                    s.nested_statement(|s| {
                        s.leading("// Check if value is positive")
                            .nested_statement(|s| s.leading("// Process payment"))
                            .nested_statement(|s| s.leading("// Double the amount"))
                            .nested_statement(|s| s.leading("// Send it back"))
                            .else_branch(|e| {
                                e.nested_statement(|s| s.leading("// No payment received"))
                            })
                    })
                })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_statement_trailing_comments() {
        let source = r#"contract Test {
    function test() public {
        uint x = 5; // Initialize x
        x = x + 1; // Increment
        require(x > 5); // Check result
    }
}"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        // Use the builder pattern for cleaner assertions
        let expected = ExpectationBuilder::new().contract("Test", |c| {
            c.function("test", |f| {
                f.visibility("public").body_statement(|s| {
                    s.nested_statement(|s| s.trailing("// Initialize x"))
                        .nested_statement(|s| s.trailing("// Increment"))
                        .nested_statement(|s| s.trailing("// Check result"))
                })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_comprehensive_recursive_comment_collection() {
        // This test verifies that we properly recurse into ALL possible statement types
        // and collect comments at every level
        let source = r#"contract ComprehensiveTest {
    function testAllStatements() public {
        // Top-level function comment
        
        // For loop with all components
        for (
            // Init comment
            uint i = 0; // Init trailing
            // Condition comment
            i < 10; // Condition trailing
            // Post comment
            i++ // Post trailing
        ) {
            // Inside for loop
            break; // Break trailing
        }
        
        // While loop
        while (true) { // While condition trailing
            // Inside while
            continue; // Continue trailing
        }
        
        // Do while
        do {
            // Inside do
            uint x = 1; // Assignment trailing
        } while (false); // Do-while trailing
        
        // If-else with multiple branches
        if (true) { // If trailing
            // Inside if
        } else if (false) { // Else-if trailing
            // Inside else-if
        } else { // Else trailing
            // Inside else
        }
        
        // Try-catch with all variants
        try externalCall() returns (uint result) { // Try trailing
            // Success case
        } catch Error(string memory reason) { // Named catch trailing
            // Error handling
        } catch Panic(uint errorCode) { // Panic catch trailing
            // Panic handling
        } catch (bytes memory lowLevelData) { // Low-level catch trailing
            // Low level handling
        } catch { // Default catch trailing
            // Fallback handling
        }
        
        // Unchecked block
        unchecked {
            // Inside unchecked
            uint y = 2; // Unchecked assignment
        }
        
        // Assembly block
        assembly {
            // YUL comment
            let z := 1 // YUL assignment trailing
        }
        
        // Emit statement
        emit SomeEvent(); // Emit trailing
        
        // Revert statements
        revert("error message"); // Revert trailing
        revert CustomError(); // Custom revert trailing
        
        // Return statement
        return 42; // Return trailing
    }
}"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        // Use the builder pattern for cleaner assertions
        let expected = ExpectationBuilder::new().contract("ComprehensiveTest", |c| {
            c.function("testAllStatements", |f| {
                f.visibility("public").body_statement(|s| {
                    s.nested_standalone_comment("// Top-level function comment")
                        .nested_statement(|s| {
                            s.leading("// For loop with all components")
                                .for_init_leading("// Init comment")
                                .for_init_trailing("// Init trailing")
                                .for_condition_leading("// Condition comment")
                                .for_condition_trailing("// Condition trailing")
                                .for_update_leading("// Post comment")
                                .for_update_trailing("// Post trailing")
                                .nested_statement(|s| {
                                    s.leading("// Inside for loop")
                                        .trailing("// Break trailing")
                                })
                        })
                        .nested_statement(|s| {
                            s.leading("// While loop")
                                .trailing("// While condition trailing")
                                .nested_statement(|s| {
                                    s.leading("// Inside while")
                                        .trailing("// Continue trailing")
                                })
                        })
                        .nested_statement(|s| {
                            s.leading("// Do while")
                                .trailing("// Do-while trailing")
                                .nested_statement(|s| {
                                    s.leading("// Inside do").trailing("// Assignment trailing")
                                })
                        })
                        .nested_statement(|s| {
                            s.leading("// If-else with multiple branches")
                                .trailing("// If trailing")
                                .nested_standalone_comment("// Inside if")
                                .else_branch(|e| {
                                    e.trailing("// Else-if trailing").nested_statement(|s| {
                                        s.nested_standalone_comment("// Inside else-if")
                                            .else_branch(|e| {
                                                e.trailing("// Else trailing")
                                                    .nested_standalone_comment("// Inside else")
                                            })
                                    })
                                })
                        })
                        .nested_statement(|s| {
                            s.leading("// Try-catch with all variants")
                                .trailing("// Try trailing")
                                .nested_standalone_comment("// Success case")
                                .catch_clause(|c| {
                                    c.trailing("// Named catch trailing")
                                        .body_standalone_comment("// Error handling")
                                })
                                .catch_clause(|c| {
                                    c.trailing("// Panic catch trailing")
                                        .body_standalone_comment("// Panic handling")
                                })
                                .catch_clause(|c| {
                                    c.trailing("// Low-level catch trailing")
                                        .body_standalone_comment("// Low level handling")
                                })
                                .catch_clause(|c| {
                                    c.trailing("// Default catch trailing")
                                        .body_standalone_comment("// Fallback handling")
                                })
                        })
                        .nested_statement(|s| {
                            s.leading("// Unchecked block").nested_statement(|s| {
                                s.leading("// Inside unchecked")
                                    .trailing("// Unchecked assignment")
                            })
                        })
                        .nested_statement(|s| {
                            s.leading("// Assembly block").yul_block(|y| {
                                y.statement(|s| {
                                    s.leading("// YUL comment")
                                        .trailing("// YUL assignment trailing")
                                })
                            })
                        })
                        .nested_statement(|s| {
                            s.leading("// Emit statement").trailing("// Emit trailing")
                        })
                        .nested_statement(|s| {
                            s.leading("// Revert statements")
                                .trailing("// Revert trailing")
                        })
                        .nested_statement(|s| s.trailing("// Custom revert trailing"))
                        .nested_statement(|s| {
                            s.leading("// Return statement")
                                .trailing("// Return trailing")
                        })
                })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_if_else_comment_collection() {
        // Simple if-else test with builder pattern
        let source = r#"contract IfElseTest {
    function test() public {
        // Check condition
        if (x > 0) {
            // Positive case
            doSomething();
        } 
        // Where does this comment go?
        // Otherwise handle small values
        else {
            // Non-positive case  
            doSomethingElse();
        }
    }
}"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        // Use the builder pattern for cleaner assertions
        let expected = ExpectationBuilder::new().contract("IfElseTest", |c| {
            c.function("test", |f| {
                f.visibility("public").body_statement(|s| {
                    s.nested_statement(|s| {
                        s.leading("// Check condition")
                            .nested_statement(|s| s.leading("// Positive case"))
                            .else_branch(|e| {
                                e.leading_many(vec![
                                    "// Where does this comment go?",
                                    "// Otherwise handle small values",
                                ])
                                .nested_statement(|s| s.leading("// Non-positive case  "))
                            })
                    })
                })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_try_catch_all_clauses() {
        let source = r#"contract TryCatchTest {
    function testTryCatch() public {
        // Try to call external function
        try externalContract.riskyOperation() returns (uint result) {
            // Success case
            emit Success(result);
        } catch Error(string memory reason) {
            // Revert with reason string
            emit Failed(reason);
        } catch Panic(uint errorCode) {
            // Panic occurred
            emit PanicOccurred(errorCode);
        } catch (bytes memory lowLevelData) {
            // Low-level error
            emit LowLevelError(lowLevelData);
        } catch {
            // Unknown error
            emit UnknownError();
        }
    }
}"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        // Use the builder pattern for cleaner assertions
        let expected = ExpectationBuilder::new().contract("TryCatchTest", |c| {
            c.function("testTryCatch", |f| {
                f.visibility("public").body_statement(|s| {
                    s.nested_statement(|s| {
                        s.leading("// Try to call external function")
                            .nested_statement(|s| s.leading("// Success case"))
                            .catch_clause(|c| {
                                c.body_statement(|s| s.leading("// Revert with reason string"))
                            })
                            .catch_clause(|c| c.body_statement(|s| s.leading("// Panic occurred")))
                            .catch_clause(|c| c.body_statement(|s| s.leading("// Low-level error")))
                            .catch_clause(|c| c.body_statement(|s| s.leading("// Unknown error")))
                    })
                })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_revert_statements_with_custom_errors() {
        let source = r#"
        // Contract with custom errors and revert statements
        contract ErrorHandlingTest {
            // Custom error definitions
            error Unauthorized(address caller); // Simple error with parameter
            error InsufficientBalance(
                // Current balance
                uint256 balance,
                // Amount requested
                uint256 requested
            );
            
            // Complex error with multiple params
            error TransferFailed(
                address from, // Sender address
                address to,   // Recipient address
                uint256 amount, // Transfer amount
                string reason // Failure reason
            );
            
            address public owner;
            mapping(address => uint256) public balances;
            
            // Constructor sets owner
            constructor() {
                owner = msg.sender;
            }
            
            // Only owner modifier
            modifier onlyOwner() {
                // Check caller is owner
                if (msg.sender != owner) {
                    // Revert with custom error
                    revert Unauthorized(msg.sender);
                }
                _; // Continue execution
            }
            
            // Transfer with various revert scenarios
            function transfer(address to, uint256 amount) public {
                // Check for zero address
                if (to == address(0)) {
                    // Revert with string message
                    revert("Cannot transfer to zero address");
                }
                
                uint256 senderBalance = balances[msg.sender];
                
                // Check sufficient balance
                if (senderBalance < amount) {
                    // Revert with custom error and params
                    revert InsufficientBalance({
                        balance: senderBalance,
                        requested: amount
                    });
                }
                
                // Attempt transfer
                balances[msg.sender] -= amount;
                balances[to] += amount;
                
                // Validate transfer
                if (balances[to] < amount) {
                    // Complex revert with multiple params
                    revert TransferFailed(
                        msg.sender, // from
                        to,         // to
                        amount,     // amount
                        "Overflow detected" // reason
                    );
                }
            }
            
            // Simple revert examples
            function simpleReverts() public pure {
                // Plain revert
                revert();
                
                // Revert with message
                revert("Simple error message");
                
                // This line is unreachable
                uint256 x = 42;
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new()
            .contract("ErrorHandlingTest", |c| {
                c.leading("// Contract with custom errors and revert statements")
                    .error("Unauthorized", |e| {
                        e.leading("// Custom error definitions")
                            .trailing("// Simple error with parameter")
                            .parameter("caller", |p| p.ty("address"))
                    })
                    .error("InsufficientBalance", |e| {
                        e.parameter("balance", |p| {
                            p.ty("uint256")
                                .leading("// Current balance")
                        })
                        .parameter("requested", |p| {
                            p.ty("uint256")
                                .leading("// Amount requested")
                        })
                    })
                    .error("TransferFailed", |e| {
                        e.leading("// Complex error with multiple params")
                            .parameter("from", |p| {
                                p.ty("address")
                                    .trailing("// Sender address")
                            })
                            .parameter("to", |p| {
                                p.ty("address")
                                    .trailing("// Recipient address")
                            })
                            .parameter("amount", |p| {
                                p.ty("uint256")
                                    .trailing("// Transfer amount")
                            })
                            .parameter("reason", |p| {
                                p.ty("string")
                                    .trailing("// Failure reason")
                            })
                    })
                    .variable("owner", |v| {
                        v.ty("address")
                            .visibility("public")
                    })
                    .variable("balances", |v| {
                        v.visibility("public")
                    })
                    .function("", |f| {
                        f.leading("// Constructor sets owner")
                            .body_statement(|s| s.nested_statement(|s| s))
                    })
                    .function("onlyOwner", |f| {
                        f.leading("// Only owner modifier")
                            .is_modifier()
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Check caller is owner")
                                        .nested_statement(|s| s.leading("// Revert with custom error"))
                                })
                                .nested_statement(|s| s.trailing("// Continue execution"))
                            })
                    })
                    .function("transfer", |f| {
                        f.leading("// Transfer with various revert scenarios")
                            .visibility("public")
                            .parameter("to", |p| p.ty("address"))
                            .parameter("amount", |p| p.ty("uint256"))
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Check for zero address")
                                        .nested_statement(|s| s.leading("// Revert with string message"))
                                })
                                .nested_statement(|s| s) // uint256 senderBalance = ...
                                .nested_statement(|s| {
                                    s.leading("// Check sufficient balance")
                                        .nested_statement(|s| s.leading("// Revert with custom error and params"))
                                })
                                .nested_statement(|s| s.leading("// Attempt transfer"))
                                .nested_statement(|s| s)
                                .nested_statement(|s| {
                                    s.leading("// Validate transfer")
                                        .nested_statement(|s| s.leading("// Complex revert with multiple params"))
                                })
                            })
                    })
                    .function("simpleReverts", |f| {
                        f.leading("// Simple revert examples")
                            .visibility("public")
                            .mutability("pure")
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Plain revert"))
                                .nested_statement(|s| s.leading("// Revert with message"))
                                .nested_statement(|s| s.leading("// This line is unreachable"))
                            })
                    })
            });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_external_function_calls_with_comments() {
        let source = r#"
        // Interface for external contract
        interface IExternalContract {
            // External transfer function
            function transfer(address to, uint256 amount) external returns (bool);
            
            // Get balance function
            function balanceOf(address account) external view returns (uint256);
        }
        
        // Contract using external calls
        contract ExternalCallTest {
            // External contract reference
            IExternalContract public externalContract;
            
            // Another external address
            address public otherContract;
            
            constructor(address _external) {
                // Initialize external contract
                externalContract = IExternalContract(_external);
            }
            
            // Function with external calls
            function performExternalCalls(address recipient) public {
                // Get current balance
                uint256 balance = externalContract.balanceOf(
                    msg.sender // Check sender's balance
                );
                
                // Perform external transfer
                bool success = externalContract.transfer(
                    recipient, // Transfer to this address
                    balance / 2 // Transfer half the balance
                );
                
                // Check transfer result
                require(success, "Transfer failed");
                
                // Call with explicit interface cast
                uint256 recipientBalance = IExternalContract(otherContract).balanceOf(
                    recipient // Get recipient's new balance
                );
                
                // Low-level call with comments
                (bool callSuccess, bytes memory data) = otherContract.call{
                    value: 1 ether, // Send 1 ETH
                    gas: 50000     // Limit gas
                }(
                    // Function selector and params
                    abi.encodeWithSelector(
                        IExternalContract.balanceOf.selector, // Function selector
                        address(this) // Check this contract's balance
                    )
                );
                
                // Staticcall example
                (bool staticSuccess, bytes memory result) = address(externalContract).staticcall(
                    // Encode the function call
                    abi.encodeWithSelector(
                        IExternalContract.balanceOf.selector, // Function selector
                        address(this) // Check this contract's balance
                    )
                );
                
                // Delegatecall example (dangerous!)
                (bool delegateSuccess, ) = otherContract.delegatecall(
                    // Delegatecall data
                    abi.encodeWithSignature(
                        "updateState(uint256)", // Function to call
                        123 // New state value
                    )
                );
                
                // Multiple external calls in expression
                uint256 totalBalance = 
                    externalContract.balanceOf(msg.sender) + // Sender balance
                    externalContract.balanceOf(address(this)) + // Contract balance
                    IExternalContract(otherContract).balanceOf(recipient); // Recipient balance
            }
            
            // Fallback with external call
            fallback() external payable {
                // Forward to external contract
                (bool sent, ) = address(externalContract).call{value: msg.value}("");
                require(sent, "Failed to forward");
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new()
            .contract("IExternalContract", |c| {
                c.leading("// Interface for external contract")
                    .contract_type("interface")
                    .function("transfer", |f| {
                        f.leading("// External transfer function")
                            .visibility("external")
                            .returns("bool")
                            .parameter("to", |p| p.ty("address"))
                            .parameter("amount", |p| p.ty("uint256"))
                    })
                    .function("balanceOf", |f| {
                        f.leading("// Get balance function")
                            .visibility("external")
                            .mutability("view")
                            .returns("uint256")
                            .parameter("account", |p| p.ty("address"))
                    })
            })
            .contract("ExternalCallTest", |c| {
                c.leading("// Contract using external calls")
                    .variable("externalContract", |v| {
                        v.leading("// External contract reference")
                            .visibility("public")
                    })
                    .variable("otherContract", |v| {
                        v.leading("// Another external address")
                            .ty("address")
                            .visibility("public")
                    })
                    .function("", |f| {
                        f.parameter("_external", |p| p.ty("address"))
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Initialize external contract"))
                            })
                    })
                    .function("performExternalCalls", |f| {
                        f.leading("// Function with external calls")
                            .visibility("public")
                            .parameter("recipient", |p| p.ty("address"))
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Get current balance"))
                                    .nested_statement(|s| s.leading("// Perform external transfer"))
                                    .nested_statement(|s| s.leading("// Check transfer result"))
                                    .nested_statement(|s| s.leading("// Call with explicit interface cast"))
                                    .nested_statement(|s| s.leading("// Low-level call with comments"))
                                    .nested_statement(|s| s.leading("// Staticcall example"))
                                    .nested_statement(|s| s.leading("// Delegatecall example (dangerous!)"))
                                    .nested_statement(|s| {
                                        s.leading("// Multiple external calls in expression")
                                            .trailing("// Recipient balance")
                                    })
                            })
                    })
                    .function("", |f| {
                        f.leading("// Fallback with external call")
                            .visibility("external")
                            .mutability("payable")
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Forward to external contract"))
                                    .nested_statement(|s| s)
                            })
                    })
            });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_super_calls_with_comments() {
        let source = r#"
        // Base contract with virtual functions
        contract Base {
            // Virtual function to be overridden
            function foo() public virtual returns (uint256) {
                return 10;
            }
            
            // Another virtual function
            function bar(uint256 x) public virtual returns (uint256) {
                return x * 2;
            }
        }
        
        // Middle contract in inheritance chain
        contract Middle is Base {
            // Override with virtual for further inheritance
            function foo() public virtual override returns (uint256) {
                // Call parent implementation
                return super.foo() + 5;
            }
        }
        
        // Derived contract using super
        contract Derived is Middle {
            // State variable for demo
            uint256 public value;
            
            // Override foo with super call
            function foo() public override returns (uint256) {
                // Get base value from parent
                uint256 baseValue = super.foo(); // Inline comment about super
                // Add our own logic
                return baseValue + 10;
            }
            
            // Override bar with super call in expression
            function bar(uint256 x) public override returns (uint256) {
                // Complex expression with super
                return super.bar(x) + // Add to parent result
                       super.foo() + // Include foo value
                       value; // And state variable
            }
            
            // Function with multiple super calls
            function combined() public returns (uint256) {
                // Call super.foo first
                uint256 a = super.foo();
                // Then call super.bar with argument
                uint256 b = super.bar(100);
                // Combine results
                return a + b;
            }
            
            // Constructor with super-like behavior
            constructor() {
                // Initialize value
                value = 42; // Default value
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new()
            .contract("Base", |c| {
                c.leading("// Base contract with virtual functions")
                    .function("foo", |f| {
                        f.leading("// Virtual function to be overridden")
                            .visibility("public")
                            .is_virtual()
                            .returns("uint256")
                            .body_statement(|s| s.nested_statement(|s| s))
                    })
                    .function("bar", |f| {
                        f.leading("// Another virtual function")
                            .visibility("public")
                            .is_virtual()
                            .parameter("x", |p| p.ty("uint256"))
                            .returns("uint256")
                            .body_statement(|s| s.nested_statement(|s| s))
                    })
            })
            .contract("Middle", |c| {
                c.leading("// Middle contract in inheritance chain")
                    .inherits("Base")
                    .function("foo", |f| {
                        f.leading("// Override with virtual for further inheritance")
                            .visibility("public")
                            .is_virtual()
                            .is_override()
                            .returns("uint256")
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Call parent implementation"))
                            })
                    })
            })
            .contract("Derived", |c| {
                c.leading("// Derived contract using super")
                    .inherits("Middle")
                    .variable("value", |v| {
                        v.leading("// State variable for demo")
                            .ty("uint256")
                            .visibility("public")
                    })
                    .function("foo", |f| {
                        f.leading("// Override foo with super call")
                            .visibility("public")
                            .is_override()
                            .returns("uint256")
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Get base value from parent")
                                        .trailing("// Inline comment about super")
                                })
                                .nested_statement(|s| {
                                    s.leading("// Add our own logic")
                                })
                            })
                    })
                    .function("bar", |f| {
                        f.leading("// Override bar with super call in expression")
                            .visibility("public")
                            .is_override()
                            .parameter("x", |p| p.ty("uint256"))
                            .returns("uint256")
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Complex expression with super")
                                        .trailing("// And state variable")
                                })
                            })
                    })
                    .function("combined", |f| {
                        f.leading("// Function with multiple super calls")
                            .visibility("public")
                            .returns("uint256")
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Call super.foo first")
                                })
                                .nested_statement(|s| {
                                    s.leading("// Then call super.bar with argument")
                                })
                                .nested_statement(|s| {
                                    s.leading("// Combine results")
                                })
                            })
                    })
                    .function("", |f| {
                        f.leading("// Constructor with super-like behavior")
                            .body_statement(|s| {
                                s.nested_statement(|s| {
                                    s.leading("// Initialize value")
                                        .trailing("// Default value")
                                })
                            })
                    })
            });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_unchecked_blocks() {
        let source = r#"
        contract UncheckedMath {
            // Function using unchecked arithmetic
            function uncheckedOperations(uint256 x, uint256 y) public pure returns (uint256) {
                // Regular checked addition
                uint256 checked = x + y;
                
                // Unchecked block for gas optimization
                unchecked {
                    // Overflow is allowed here
                    uint256 result = x + y;
                    
                    // Multiple operations
                    result = result * 2; // No overflow check
                    result = result - 1; // No underflow check
                    
                    // Nested unchecked (redundant but valid)
                    unchecked {
                        // Still unchecked
                        result = result / 2;
                    }
                    
                    return result;
                }
            }
            
            // Complex unchecked with control flow
            function complexUnchecked(uint256[] memory values) public pure returns (uint256) {
                uint256 sum = 0;
                
                // Unchecked loop for efficiency
                unchecked {
                    // No overflow checks in loop
                    for (uint256 i = 0; i < values.length; i++) {
                        // Add without overflow check
                        sum += values[i];
                        
                        // Conditional in unchecked
                        if (sum > 1000) {
                            // Early exit
                            break;
                        }
                    }
                }
                
                return sum;
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new()
            .contract("UncheckedMath", |c| {
                c.function("uncheckedOperations", |f| {
                    f.leading("// Function using unchecked arithmetic")
                        .visibility("public")
                        .mutability("pure")
                        .returns("uint256")
                        .parameter("x", |p| p.ty("uint256"))
                        .parameter("y", |p| p.ty("uint256"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s.leading("// Regular checked addition"))
                                .nested_statement(|s| {
                                    s.leading("// Unchecked block for gas optimization")
                                        .nested_statement(|s| s.leading("// Overflow is allowed here"))
                                        .nested_statement(|s| {
                                            s.leading("// Multiple operations")
                                                .trailing("// No overflow check")
                                        })
                                        .nested_statement(|s| s.trailing("// No underflow check"))
                                        .nested_statement(|s| {
                                            s.leading("// Nested unchecked (redundant but valid)")
                                                .nested_statement(|s| {
                                                    s.leading("// Still unchecked")
                                                })
                                        })
                                        .nested_statement(|s| s)
                                })
                        })
                })
                .function("complexUnchecked", |f| {
                    f.leading("// Complex unchecked with control flow")
                        .visibility("public")
                        .mutability("pure")
                        .returns("uint256")
                        .parameter("values", |p| p.ty("uint256[]").storage("memory"))
                        .body_statement(|s| {
                            s.nested_statement(|s| s)
                                .nested_statement(|s| {
                                    s.leading("// Unchecked loop for efficiency")
                                        .nested_statement(|s| {
                                            s.leading("// No overflow checks in loop")
                                                .nested_statement(|s| {
                                                    s.leading("// Add without overflow check")
                                                })
                                                .nested_statement(|s| {
                                                    s.leading("// Conditional in unchecked")
                                                        .nested_statement(|s| s.leading("// Early exit"))
                                                })
                                        })
                                })
                                .nested_statement(|s| s)
                        })
                })
            });

        expected.assert_matches(&collected);
    }
}