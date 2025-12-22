#[cfg(test)]
mod tests {
    use crate::collector::collect_source_unit;
    use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};
    use solang_parser::parse;

    #[test]
    fn test_empty_contracts_with_comments() {
        let source = r#"
        // Empty contract with only comments
        contract EmptyWithComments {
            // This contract is empty
            // But has several comments
            
            /* Block comment
               spanning multiple lines
               in an empty contract */
            
            // Another comment
        }
        
        // Interface with only comments
        interface EmptyInterface {
            // No functions defined
            // Just documentation
        }
        
        // Abstract contract
        abstract contract EmptyAbstract {
            // TODO: Add implementation
            // This is a placeholder
            
            /* Design notes:
             * - Should implement ERC20
             * - Needs access control
             */
        }
        
        // Library with comments
        library EmptyLibrary {
            // Utility functions will go here
            // Currently empty for testing
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("EmptyWithComments", |c| {
                c.leading("// Empty contract with only comments")
                    .standalone_comment("// This contract is empty")
                    .standalone_comment("// But has several comments")
                    .standalone_comment("/* Block comment\n               spanning multiple lines\n               in an empty contract */")
                    .standalone_comment("// Another comment")
            })
            .contract("EmptyInterface", |c| {
                c.leading("// Interface with only comments")
                    .standalone_comment("// No functions defined")
                    .standalone_comment("// Just documentation")
            })
            .contract("EmptyAbstract", |c| {
                c.leading("// Abstract contract")
                    .standalone_comment("// TODO: Add implementation")
                    .standalone_comment("// This is a placeholder")
                    .standalone_comment("/* Design notes:\n             * - Should implement ERC20\n             * - Needs access control\n             */")
            })
            .contract("EmptyLibrary", |c| {
                c.leading("// Library with comments")
                    .standalone_comment("// Utility functions will go here")
                    .standalone_comment("// Currently empty for testing")
            });
        expected.assert_matches(&collected);
    }

    #[test]
    fn test_unicode_characters_in_comments() {
        let source = r#"
        // Contract with Unicode comments 🚀
        contract UnicodeTest {
            // Mathematical symbols: ∑ ∏ ∫ ∂ ∇ ∞
            uint256 public constant PI = 314159;
            
            // Greek letters: α β γ δ ε ζ η θ
            mapping(address => uint256) public balances;
            
            // Currency symbols: $ € £ ¥ ₹ ₿
            function transfer(
                address to, // → recipient
                uint256 amount // ← amount to send
            ) public {
                // Check balance ≥ amount
                require(balances[msg.sender] >= amount, "Insufficient balance");
                
                // Transfer tokens ✓
                balances[msg.sender] -= amount;
                balances[to] += amount;
            }
            
            /* Chinese characters: 你好世界
             * Japanese: こんにちは世界
             * Korean: 안녕하세요 세계
             * Arabic: مرحبا بالعالم
             * Hebrew: שלום עולם
             */
            
            // Emojis: 🔥 💎 🎉 🎨 🔒 🔑 ⚡ 🌟
            event TokenTransfer(
                address indexed from, // 👤 sender
                address indexed to,   // 👥 receiver  
                uint256 value        // 💰 amount
            );
            
            // Box drawing: ┌─┬─┐ │ ├─┼─┤ └─┴─┘
            struct Config {
                bool active;    // ✅ or ❌
                uint256 limit;  // ⚠️ maximum allowed
            }
            
            // Mathematical operators: ≤ ≥ ≠ ≈ ± × ÷
            modifier onlyOwner() {
                require(msg.sender == owner, "Not authorized ⛔");
                _; // ← continue execution
            }
            
            // Arrows and symbols: ⇒ ⇐ ⇔ ⇄ ↑ ↓ ↔ ↕
            function calculate(uint256 x) pure public returns (uint256) {
                // x² + 2x + 1 = (x + 1)²
                return x * x + 2 * x + 1;
            }
        }
        "#;
        let (source_unit, comments) = parse(source, 0).expect("Failed to parse");
        let collected = collect_source_unit(&source_unit, &comments, source);
        let expected = ExpectationBuilder::new()
            .contract("UnicodeTest", |c| {
                c.leading("// Contract with Unicode comments 🚀")
                    .variable("PI", |v| {
                        v.ty("uint256")
                            .visibility("public")
                            .constant()
                            .leading("// Mathematical symbols: ∑ ∏ ∫ ∂ ∇ ∞")
                    })
                    .variable("balances", |v| {
                        v.visibility("public")
                            .leading("// Greek letters: α β γ δ ε ζ η θ")
                    })
                    .function("transfer", |f| {
                        f.leading("// Currency symbols: $ € £ ¥ ₹ ₿")
                            .visibility("public")
                            .parameter("to", |p| {
                                p.ty("address")
                                    .trailing("// → recipient")
                            })
                            .parameter("amount", |p| {
                                p.ty("uint256")
                                    .trailing("// ← amount to send")
                            })
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// Check balance ≥ amount"))
                                    .nested_statement(|s| s.leading("// Transfer tokens ✓"))
                                    .nested_statement(|s| s)
                            })
                    })
                    .standalone_comment("/* Chinese characters: 你好世界\n             * Japanese: こんにちは世界\n             * Korean: 안녕하세요 세계\n             * Arabic: مرحبا بالعالم\n             * Hebrew: שלום עולם\n             */")
                    .event("TokenTransfer", |e| {
                        e.leading("// Emojis: 🔥 💎 🎉 🎨 🔒 🔑 ⚡ 🌟")
                            .parameter("from", |p| {
                                p.indexed()
                                    .trailing("// 👤 sender")
                            })
                            .parameter("to", |p| {
                                p.indexed()
                                    .trailing("// 👥 receiver  ")
                            })
                            .parameter("value", |p| {
                                p.trailing("// 💰 amount")
                            })
                    })
                    .struct_def("Config", |s| {
                        s.leading("// Box drawing: ┌─┬─┐ │ ├─┼─┤ └─┴─┘")
                            .field("active", |f| {
                                f.ty("bool")
                                    .trailing("// ✅ or ❌")
                            })
                            .field("limit", |f| {
                                f.ty("uint256")
                                    .trailing("// ⚠️ maximum allowed")
                            })
                    })
                    .function("onlyOwner", |f| {
                        f.leading("// Mathematical operators: ≤ ≥ ≠ ≈ ± × ÷")
                            .is_modifier()
                            .body_statement(|s| {
                                s.nested_statement(|s| s)
                                    .nested_statement(|s| s.trailing("// ← continue execution"))
                            })
                    })
                    .function("calculate", |f| {
                        f.leading("// Arrows and symbols: ⇒ ⇐ ⇔ ⇄ ↑ ↓ ↔ ↕")
                            .parameter("x", |p| p.ty("uint256"))
                            .mutability("pure")
                            .visibility("public")
                            .returns("uint256")
                            .body_statement(|s| {
                                s.nested_statement(|s| s.leading("// x² + 2x + 1 = (x + 1)²"))
                            })
                    })
            });
        expected.assert_matches(&collected);
    }
}