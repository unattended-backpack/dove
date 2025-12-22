//! Test builder module for creating test expectations
//!
//! This module provides a fluent API for building test expectations that can be
//! matched against collected AST elements with their associated comments.

#![allow(missing_docs)]

use solang_parser::pt::{
    Comment, Expression, ImportPath, Mutability, Type, VersionComparator, VersionOp, Visibility,
};

// Re-export all the expectation types from submodules
pub use builder::ExpectationBuilder;
pub use events_errors::{
    ErrorExpectation, ErrorParameterExpectation, EventExpectation, EventParameterExpectation,
};
pub use functions::{FunctionExpectation, FunctionParameterExpectation};
pub use statements::{CatchClauseExpectation, ElseBranchExpectation, StatementExpectation};
pub use top_level::{ContractExpectation, ImportExpectation, PragmaExpectation, UsingExpectation};
pub use types::{
    EnumExpectation, EnumValueExpectation, FieldExpectation, StructExpectation, TypeExpectation,
};
pub use variables::{TypeCommentsBuilder, TypeCommentsExpectation, VariableExpectation};
pub use yul::{YulBlockExpectation, YulStatementExpectation, YulSwitchCaseExpectation};

// Submodules
mod builder;
mod events_errors;
mod functions;
mod statements;
mod top_level;
mod types;
mod variables;
mod yul;

// Base trait for common expectation methods
pub trait CommentExpectation: Sized {
    fn leading_comments_mut(&mut self) -> &mut Vec<String>;
    fn trailing_comments_mut(&mut self) -> &mut Vec<String>;

    fn leading(mut self, comment: &str) -> Self {
        self.leading_comments_mut().push(comment.to_string());
        self
    }

    fn trailing(mut self, comment: &str) -> Self {
        self.trailing_comments_mut().push(comment.to_string());
        self
    }

    fn leading_many(mut self, comments: Vec<&str>) -> Self {
        for comment in comments {
            self.leading_comments_mut().push(comment.to_string());
        }
        self
    }

    fn trailing_many(mut self, comments: Vec<&str>) -> Self {
        for comment in comments {
            self.trailing_comments_mut().push(comment.to_string());
        }
        self
    }
}

/// Helper function to assert comment text matches
pub(crate) fn assert_comment_text(comment: &Comment, expected: &str, context: &str) {
    let text = match comment {
        Comment::Line(_, text) => text,
        Comment::Block(_, text) => text,
        Comment::DocLine(_, text) => text,
        Comment::DocBlock(_, text) => text,
    };
    assert_eq!(text, expected, "{}: comment text mismatch", context);
}

/// Helper function to assert comments match expected
pub(crate) fn assert_comments(comments: &[Comment], expected: &[String], context: &str) {
    assert_eq!(
        comments.len(),
        expected.len(),
        "{}: expected {} comments, found {}",
        context,
        expected.len(),
        comments.len()
    );

    for (i, (comment, expected_text)) in comments.iter().zip(expected.iter()).enumerate() {
        assert_comment_text(
            comment,
            expected_text,
            &format!("{}.comment[{}]", context, i),
        );
    }
}

/// Helper function for formatting version requirements (copied from toplevel.rs)
pub(crate) fn format_version_requirement(req: &VersionComparator) -> String {
    match req {
        VersionComparator::Plain { version, .. } => version.join("."),
        VersionComparator::Operator { op, version, .. } => {
            format!("{}{}", format_version_op(op), version.join("."))
        }
        VersionComparator::Or { left, right, .. } => {
            format!(
                "{} || {}",
                format_version_requirement(left),
                format_version_requirement(right)
            )
        }
        VersionComparator::Range { from, to, .. } => {
            format!("{} - {}", from.join("."), to.join("."))
        }
    }
}

/// Helper function for formatting version operators (copied from toplevel.rs)
pub(crate) fn format_version_op(op: &VersionOp) -> &'static str {
    match op {
        VersionOp::Exact => "",
        VersionOp::Greater => ">",
        VersionOp::GreaterEq => ">=",
        VersionOp::Less => "<",
        VersionOp::LessEq => "<=",
        VersionOp::Caret => "^",
        VersionOp::Tilde => "~",
        VersionOp::Wildcard => "*",
    }
}

/// Helper function for formatting visibility
pub(crate) fn format_visibility(vis: &Visibility) -> &'static str {
    match vis {
        Visibility::Public(_) => "public",
        Visibility::Private(_) => "private",
        Visibility::Internal(_) => "internal",
        Visibility::External(_) => "external",
    }
}

/// Helper function for formatting mutability
pub(crate) fn format_mutability(mutability: &Mutability) -> &'static str {
    match mutability {
        Mutability::Pure(_) => "pure",
        Mutability::View(_) => "view",
        Mutability::Constant(_) => "constant",
        Mutability::Payable(_) => "payable",
    }
}

/// Helper function to extract type string from an Expression
pub(crate) fn extract_type_string(expr: &Expression) -> String {
    match expr {
        // Simple identifier (user-defined type like ValueStruct)
        Expression::Variable(ident) => ident.name.clone(),

        // Built-in type
        Expression::Type(_, ty) => match ty {
            Type::Uint(size) => format!("uint{}", size),
            Type::Int(size) => format!("int{}", size),
            Type::Bool => "bool".to_string(),
            Type::Address => "address".to_string(),
            Type::AddressPayable => "address payable".to_string(),
            Type::String => "string".to_string(),
            Type::Bytes(size) => format!("bytes{}", size),
            Type::DynamicBytes => "bytes".to_string(),
            Type::Mapping { key, value, .. } => {
                let key_type = extract_type_string(key);
                let value_type = extract_type_string(value);
                format!("mapping({} => {})", key_type, value_type)
            }
            Type::Function { .. } => {
                // Function types are complex - for now just return a placeholder
                "function_type".to_string()
            }
            _ => "complex_type".to_string(),
        },

        // Array type (e.g., uint256[], address[], ValueStruct[])
        Expression::ArraySubscript(_, base_expr, size_expr) => {
            let base_type = extract_type_string(base_expr);
            match size_expr {
                Some(size) => {
                    // Fixed-size array like uint256[10]
                    if let Expression::NumberLiteral(_, num, _, _) = &**size {
                        format!("{}[{}]", base_type, num)
                    } else {
                        format!("{}[<expr>]", base_type)
                    }
                }
                None => {
                    // Dynamic array like uint256[]
                    format!("{}[]", base_type)
                }
            }
        }
        _ => "complex_type".to_string(),
    }
}

/// Helper function to extract import path string from ImportPath
pub(crate) fn extract_import_path(import_path: &ImportPath) -> String {
    match import_path {
        ImportPath::Filename(string_lit) => string_lit.string.clone(),
        ImportPath::Path(ident_path) => ident_path
            .identifiers
            .iter()
            .map(|id| id.name.clone())
            .collect::<Vec<_>>()
            .join("."),
    }
}
