use crate::collector::*;
use solang_parser::parse;
use crate::collector::test_builder::{CommentExpectation, ExpectationBuilder};

#[test]
fn test_struct_field_comments() {
    let source = r#"contract Test {
    struct UserData {
        // User's ethereum address
        address userAddress; // Must be non-zero
        
        // User's balance in wei
        uint256 balance;
        
        // Whether the user is active
        bool isActive; // Can be toggled by admin
    }
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    // Use the builder pattern for cleaner assertions
    let expected = ExpectationBuilder::new().contract("Test", |c| {
        c.struct_def("UserData", |s| {
            s.field("userAddress", |f| {
                f.ty("address")
                    .leading("// User's ethereum address")
                    .trailing("// Must be non-zero")
            })
            .field("balance", |f| {
                f.ty("uint256").leading("// User's balance in wei")
            })
            .field("isActive", |f| {
                f.ty("bool")
                    .leading("// Whether the user is active")
                    .trailing("// Can be toggled by admin")
            })
        })
    });

    expected.assert_matches(&collected);
}

#[test]
fn test_enum_value_comments() {
    let source = r#"contract Test {
    enum Status {
        // Not yet started
        Pending, // Default state
        
        // Currently in progress
        Active,
        
        // Successfully completed
        Completed, // Final state
        
        // Cancelled by user
        Cancelled
    }
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    // Use the builder pattern for cleaner assertions
    let expected = ExpectationBuilder::new().contract("Test", |c| {
        c.enum_def("Status", |e| {
            e.value("Pending", |v| {
                v.leading("// Not yet started").trailing("// Default state")
            })
            .value("Active", |v| v.leading("// Currently in progress"))
            .value("Completed", |v| {
                v.leading("// Successfully completed")
                    .trailing("// Final state")
            })
            .value("Cancelled", |v| v.leading("// Cancelled by user"))
        })
    });

    expected.assert_matches(&collected);
}

#[test]
fn test_event_parameter_comments() {
    let source = r#"contract Test {
    event Transfer(
        // The address sending tokens
        address indexed from, // Must be non-zero
        
        // The address receiving tokens
        address indexed to, // Can be zero for burns
        
        // The amount being transferred
        uint256 value // In wei
    );
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    // Use the builder pattern for cleaner assertions
    let expected = ExpectationBuilder::new().contract("Test", |c| {
        c.event("Transfer", |e| {
            e.parameter("from", |p| {
                p.ty("address")
                    .indexed()
                    .leading("// The address sending tokens")
                    .trailing("// Must be non-zero")
            })
            .parameter("to", |p| {
                p.ty("address")
                    .indexed()
                    .leading("// The address receiving tokens")
                    .trailing("// Can be zero for burns")
            })
            .parameter("value", |p| {
                p.ty("uint256")
                    // Not indexed
                    .leading("// The amount being transferred")
                    .trailing("// In wei")
            })
        })
    });

    expected.assert_matches(&collected);
}

#[test]
fn test_error_parameter_comments() {
    let source = r#"contract Test {
    error InsufficientBalance(
        // The account that tried to send
        address account, // Sender address
        
        // The balance they had
        uint256 available, // In wei
        
        // The amount they tried to send
        uint256 required // Must be <= available
    );
}"#;

    let (source_unit, comments) = parse(source, 0).expect("Failed to parse source");
    let collected = collect_source_unit(&source_unit, &comments, source);

    // Use the builder pattern for cleaner assertions
    let expected = ExpectationBuilder::new().contract("Test", |c| {
        c.error("InsufficientBalance", |e| {
            e.parameter("account", |p| {
                p.ty("address")
                    .leading("// The account that tried to send")
                    .trailing("// Sender address")
            })
            .parameter("available", |p| {
                p.ty("uint256")
                    .leading("// The balance they had")
                    .trailing("// In wei")
            })
            .parameter("required", |p| {
                p.ty("uint256")
                    .leading("// The amount they tried to send")
                    .trailing("// Must be <= available")
            })
        })
    });

    expected.assert_matches(&collected);
}