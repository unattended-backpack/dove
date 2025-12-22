#[cfg(test)]
mod tests {
    use crate::collector::collect_source_unit;
    use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
    use solang_parser::parse;

    #[test]
    fn test_yul_block_comments() {
        // This test verifies that comments within Yul/assembly blocks are properly collected
        // including both standalone and statement-associated comments

        let source = r#"
import { ICrossL2Inbox, Identifier } from "interfaces/L2/ICrossL2Inbox.sol";
import { ISystemConfigInterop } from "interfaces/L2/ISystemConfigInterop.sol";

/// @title EventLogger
/// @notice EventLogger is a util contract to emit log events, primarily for
/// integration testing.
contract EventLogger {

  /// @notice Emits an event log with the given number of topics and the given
  /// data, which, when properly parsed, represents a supervisor Log event.
  function emitLog(
    address _origin,
    bytes32[] calldata _topics,
    bytes calldata _data
  ) external {
    assembly {
      let length := mload(_data)  // Get data length
      let data := add(_data, 0x20)  // Skip length prefix
      let _topicsLength := shl(5, _topics.length)
      let i := 0
      
      // Switch based on number of topics
      switch _topics.length
      case 0 { 
        // No topics
        log0(data, length) 
      }
      case 1 { 
        // Single topic
        log1(data, length, calldataload(add(_topics.offset, i))) 
      }
      case 2 {
        // Two topics
        log2(
          data,
          length,
          calldataload(add(_topics.offset, i)),
          calldataload(add(_topics.offset, add(i, 0x20)))
        )
      }
      case 3 {
        // Three topics
        log3(
          data,
          length,
          calldataload(add(_topics.offset, i)),
          calldataload(add(_topics.offset, add(i, 0x20))),
          calldataload(add(_topics.offset, add(i, 0x40)))
        )
      }
      default {
        // More than 3 topics - YUL switch requires default case
        log4(
          data,
          length,
          calldataload(add(_topics.offset, i)),
          calldataload(add(_topics.offset, add(i, 0x20))),
          calldataload(add(_topics.offset, add(i, 0x40))),
          calldataload(add(_topics.offset, add(i, 0x60)))
        )
      }
    }
  }
}"#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
        let collected = collect_source_unit(&source_unit, &comments, source);

        // Use the builder pattern for cleaner assertions
        let expected = ExpectationBuilder::new()
            .import(|i| {
                i.path("interfaces/L2/ICrossL2Inbox.sol")
                    .symbol("ICrossL2Inbox", None)
                    .symbol("Identifier", None)
            }) // First import
            .import(|i| {
                i.path("interfaces/L2/ISystemConfigInterop.sol")
                    .symbol("ISystemConfigInterop", None)
            }) // Second import
            .contract("EventLogger", |c| {
                c.leading_many(vec![
                    "/// @title EventLogger",
                    "/// @notice EventLogger is a util contract to emit log events, primarily for",
                    "/// integration testing.",
                ])
                .function("emitLog", |f| {
                    f.visibility("external")
                        .parameter("_origin", |p| p.ty("address"))
                        .parameter("_topics", |p| {
                            p.ty("bytes32[]").storage_location("calldata")
                        })
                        .parameter("_data", |p| p.ty("bytes").storage_location("calldata"))
                        .leading_many(vec![
                            "/// @notice Emits an event log with the given number of topics and the given",
                            "/// data, which, when properly parsed, represents a supervisor Log event.",
                        ])
                        .body_statement(|s| {
                            // The function body is a Block statement containing the assembly
                            s.nested_statement(|s| {
                                s.yul_block(|y| {
                                y.statement(|s| s.trailing("// Get data length"))
                                    .statement(|s| s.trailing("// Skip length prefix"))
                                    .statement(|s| s)
                                    .statement(|s| s)
                                    .statement(|s| {
                                        s.leading("// Switch based on number of topics")
                                            .switch_case(|c| {
                                            c.body_statement(|s| s.leading("// No topics"))
                                        })
                                        .switch_case(|c| {
                                            c.body_statement(|s| s.leading("// Single topic"))
                                        })
                                        .switch_case(|c| {
                                            c.body_statement(|s| s.leading("// Two topics"))
                                        })
                                        .switch_case(|c| {
                                            c.body_statement(|s| s.leading("// Three topics"))
                                        })
                                        .switch_case(|c| {
                                            c.body_statement(|s| s.leading("// More than 3 topics - YUL switch requires default case"))
                                        })
                                    })
                                })
                            })
                        })
                })
            });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_yul_functions_with_comments() {
        let source = r#"
        contract YulFunctionTest {
            function assemblyTest() public pure returns (uint256 result) {
                assembly {
                    // Simple Yul block comment
                    let x := 10
                    
                    // Another comment
                    let y := 3
                    
                    // Calculate result
                    result := add(x, y)
                }
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("YulFunctionTest", |c| {
            c.function("assemblyTest", |f| {
                f.visibility("public")
                    .mutability("pure")
                    .returns("uint256")
                    .body_statement(|s| {
                        // The assembly block is a nested statement
                        s.nested_statement(|s| {
                            s.yul_block(|y| {
                                y.statement(|s| s.leading("// Simple Yul block comment"))
                                    .statement(|s| s.leading("// Another comment"))
                                    .statement(|s| s.leading("// Calculate result"))
                            })
                        })
                    })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_yul_for_loops_with_comments() {
        let source = r#"
        contract YulForLoopTest {
            function loopTest() public pure returns (uint256 sum) {
                assembly {
                    // Initialize sum
                    sum := 0
                    
                    // Loop from 1 to 10
                    for 
                        // Initialize counter
                        { let i := 1 } 
                        // Loop condition
                        lt(i, 11) 
                        // Update counter
                        { i := add(i, 1) }
                    {
                        // Add to sum
                        sum := add(sum, i)
                    }
                    
                    // Another for loop example
                    for { 
                        // Init multiple variables
                        let j := 0 
                        let k := 100 
                    } 
                    // Complex condition
                    and(lt(j, 10), gt(k, 90)) 
                    { 
                        // Update both
                        j := add(j, 1)
                        k := sub(k, 1)
                    }
                    {
                        // Loop body
                        sum := add(sum, j)
                    }
                }
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("YulForLoopTest", |c| {
            c.function("loopTest", |f| {
                f.visibility("public")
                    .mutability("pure")
                    .returns("uint256")
                    .body_statement(|s| {
                        // The assembly block
                        s.nested_statement(|s| {
                            s.yul_block(|y| {
                                y.statement(|s| s.leading("// Initialize sum"))
                                    .statement(|s| {
                                        s.leading("// Loop from 1 to 10")
                                            // For loop comments appear as standalone comments in the loop
                                            .nested_standalone_comment("// Initialize counter")
                                            .nested_standalone_comment("// Loop condition")
                                            .nested_standalone_comment("// Update counter")
                                            .nested_statement(|s| s.leading("// Add to sum"))
                                    })
                                    .statement(|s| {
                                        s.leading("// Another for loop example")
                                            .nested_standalone_comment("// Init multiple variables")
                                            .nested_standalone_comment("// Complex condition")
                                            .nested_standalone_comment("// Update both")
                                            .nested_statement(|s| s.leading("// Loop body"))
                                    })
                            })
                        })
                    })
            })
        });

        expected.assert_matches(&collected);
    }

    #[test]
    fn test_yul_switch_with_comments() {
        let source = r#"
        contract YulSwitchTest {
            function switchTest(uint256 value) public pure returns (string memory) {
                assembly {
                    // Allocate memory for result
                    let result := mload(0x40)
                    
                    // Switch on input value
                    switch value
                    // Zero case
                    case 0 {
                        // Handle zero
                        mstore(result, 4)
                        mstore(add(result, 0x20), "zero")
                    }
                    // One case
                    case 1 {
                        // Handle one
                        mstore(result, 3)
                        mstore(add(result, 0x20), "one")
                    }
                    // Default case
                    default {
                        // Handle other values
                        mstore(result, 5)
                        mstore(add(result, 0x20), "other")
                    }
                    
                    // Update free memory pointer
                    mstore(0x40, add(result, 0x40))
                }
            }
        }
        "#;

        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);

        let expected = ExpectationBuilder::new().contract("YulSwitchTest", |c| {
            c.function("switchTest", |f| {
                f.visibility("public")
                    .mutability("pure")
                    .returns("string")
                    .parameter("value", |p| p.ty("uint256"))
                    .body_statement(|s| {
                        // The assembly block
                        s.nested_statement(|s| {
                            s.yul_block(|y| {
                                y.statement(|s| s.leading("// Allocate memory for result"))
                                    .statement(|s| {
                                        s.leading("// Switch on input value")
                                            .switch_case(|c| {
                                                c.leading("// Zero case")
                                                    .body_statement(|s| s.leading("// Handle zero"))
                                                    .body_statement(|s| s)
                                            })
                                            .switch_case(|c| {
                                                c.leading("// One case")
                                                    .body_statement(|s| s.leading("// Handle one"))
                                                    .body_statement(|s| s)
                                            })
                                            .switch_case(|c| {
                                                c.leading("// Default case")
                                                    .body_statement(|s| s.leading("// Handle other values"))
                                                    .body_statement(|s| s)
                                            })
                                    })
                                    .statement(|s| s.leading("// Update free memory pointer"))
                            })
                        })
                    })
            })
        });

        expected.assert_matches(&collected);
    }
}