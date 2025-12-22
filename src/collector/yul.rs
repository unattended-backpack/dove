//! YUL-related collection functions

use crate::collector::comments::{
    find_trailing_comments, separate_internal_and_after_comments, take_last_comment_group_before,
};
use crate::collector::utils::{get_loc_end, get_loc_range, get_loc_start};
use super::model::*;
use solang_parser::pt::*;
use solang_parser::helpers::CodeLocation;

/// Get location range for a YUL statement
fn get_yul_statement_location(stmt: &YulStatement) -> (usize, usize) {
    match stmt {
        YulStatement::Assign(loc, _, _) => get_loc_range(loc),
        YulStatement::VariableDeclaration(loc, _, _) => get_loc_range(loc),
        YulStatement::If(loc, _, _) => get_loc_range(loc),
        YulStatement::For(yul_for) => get_loc_range(&yul_for.loc),
        YulStatement::Switch(yul_switch) => get_loc_range(&yul_switch.loc),
        YulStatement::Leave(loc) => get_loc_range(loc),
        YulStatement::Break(loc) => get_loc_range(loc),
        YulStatement::Continue(loc) => get_loc_range(loc),
        YulStatement::Block(block) => get_loc_range(&block.loc),
        YulStatement::FunctionDefinition(func_def) => get_loc_range(&func_def.loc),
        YulStatement::FunctionCall(func_call) => get_loc_range(&func_call.loc),
        YulStatement::Error(loc) => get_loc_range(loc),
    }
}

/// Get location range for a YUL switch case
fn get_yul_case_location(case: &YulSwitchOptions) -> (usize, usize) {
    match case {
        YulSwitchOptions::Case(loc, _expr, block) => {
            let start = get_loc_start(loc);
            let end = get_loc_end(&block.loc);
            (start, end)
        }
        YulSwitchOptions::Default(loc, block) => {
            let start = get_loc_start(loc);
            let end = get_loc_end(&block.loc);
            (start, end)
        }
    }
}

/// Collect YUL statements with their associated comments
fn collect_yul_statements(
    statements: &[YulStatement],
    comments: &[Comment],
    source: &str,
) -> (Vec<CommentedYulStatement>, Vec<Comment>) {
    let mut collected_statements = Vec::new();
    let mut remaining_comments = comments.to_vec();

    for stmt in statements {
        let (start, end) = get_yul_statement_location(stmt);

        // Take the last comment group before this statement
        let (leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // Find trailing comments - special handling for if statements
        let trailing_comments = match stmt {
            YulStatement::If(loc, _cond, block) => {
                // For YUL if statements, find comments after the opening brace on the same line
                // In YUL: "if 1 { // comment", the comment is after the "{"
                let if_start = get_loc_start(loc);
                let block_start = get_loc_start(&block.loc);

                // Find the end of the line containing the if statement start
                let line_end = source[if_start..]
                    .find('\n')
                    .map(|i| if_start + i)
                    .unwrap_or(source.len());

                // Find comments after the block start but on the same line
                let mut trailing = Vec::new();
                for comment in &remaining_comments {
                    let comment_start = get_loc_start(&comment.loc());
                    if comment_start > block_start && comment_start < line_end {
                        trailing.push(comment.clone());
                    }
                }
                trailing
            }
            _ => {
                // For other statements, use the default behavior
                find_trailing_comments(&remaining_comments, source, end)
            }
        };

        // Separate comments into: internal, trailing, and everything else
        let (internal_comments, after_statement, _) = separate_internal_and_after_comments(
            remaining_comments,
            &trailing_comments,
            start,
            end,
        );

        // Update remaining comments (everything after this statement, excluding trailing)
        remaining_comments = after_statement;

        // Handle nested YUL statements recursively
        let (processed_stmt, nested_statements, nested_standalone, switch_cases) = match stmt {
            YulStatement::Block(block) => {
                // Recursively process block statements
                let (nested, standalone) =
                    collect_yul_statements(&block.statements, &internal_comments, source);
                (stmt.clone(), Some(nested), Some(standalone), None)
            }
            YulStatement::If(_loc, _cond, block) => {
                // Process the then block
                let (nested, standalone) =
                    collect_yul_statements(&block.statements, &internal_comments, source);
                (stmt.clone(), Some(nested), Some(standalone), None)
            }
            YulStatement::For(yul_for) => {
                // Process the for execution block
                let (nested, standalone) = collect_yul_statements(
                    &yul_for.execution_block.statements,
                    &internal_comments,
                    source,
                );
                (stmt.clone(), Some(nested), Some(standalone), None)
            }
            YulStatement::Switch(yul_switch) => {
                // Collect all switch cases with their comments
                let switch_cases = collect_yul_cases(
                    &yul_switch.cases,
                    &yul_switch.default,
                    &internal_comments,
                    source,
                );

                // For compatibility with the old structure, we still extract nested statements
                // This allows the existing logic to continue working
                let mut all_nested = Vec::new();
                let mut all_standalone = Vec::new();

                for case in &switch_cases {
                    all_nested.extend(case.body_statements.clone());
                    all_standalone.extend(case.body_standalone_comments.clone());
                }

                (
                    stmt.clone(),
                    Some(all_nested),
                    Some(all_standalone),
                    Some(switch_cases),
                )
            }
            YulStatement::FunctionDefinition(func_def) => {
                // Process the function body block
                let (nested, standalone) =
                    collect_yul_statements(&func_def.body.statements, &internal_comments, source);
                (stmt.clone(), Some(nested), Some(standalone), None)
            }
            _ => (stmt.clone(), None, None, None), // For other YUL statements, no nested processing needed
        };

        collected_statements.push(CommentedYulStatement {
            statement: processed_stmt,
            leading_comments,
            trailing_comments,
            nested_statements,
            nested_standalone_comments: nested_standalone,
            switch_cases,
        });
    }

    // Return collected statements and any remaining comments as standalone
    (collected_statements, remaining_comments)
}

/// Collect YUL switch cases with their associated comments
fn collect_yul_cases(
    cases: &[YulSwitchOptions],
    default_case: &Option<YulSwitchOptions>,
    comments: &[Comment],
    source: &str,
) -> Vec<CommentedYulCase> {
    let mut collected_cases = Vec::new();
    let mut remaining_comments = comments.to_vec();

    // Process regular cases
    for case in cases {
        let (start, end) = get_yul_case_location(case);

        // Take the last comment group before this case
        let (leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // Find trailing comments after the entire case (including its block)
        let trailing_comments = find_trailing_comments(&remaining_comments, source, end);

        // Separate comments into: internal (within case block), trailing, and everything else
        let (internal_comments, after_case, _) = separate_internal_and_after_comments(
            remaining_comments,
            &trailing_comments,
            start,
            end,
        );

        // Update remaining comments
        remaining_comments = after_case;

        // Process the case block body
        let (body_statements, body_standalone_comments) = match case {
            YulSwitchOptions::Case(_, _, block) => {
                collect_yul_statements(&block.statements, &internal_comments, source)
            }
            YulSwitchOptions::Default(_, block) => {
                collect_yul_statements(&block.statements, &internal_comments, source)
            }
        };

        collected_cases.push(CommentedYulCase {
            case: case.clone(),
            leading_comments,
            trailing_comments,
            body_statements,
            body_standalone_comments,
        });
    }

    // Process default case if it exists
    if let Some(default) = default_case {
        let (start, end) = get_yul_case_location(default);

        // Take the last comment group before the default case
        let (leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // Find trailing comments after the entire default case
        let trailing_comments = find_trailing_comments(&remaining_comments, source, end);

        // Separate comments
        let (internal_comments, _after_default, _) = separate_internal_and_after_comments(
            remaining_comments,
            &trailing_comments,
            start,
            end,
        );

        // Process the default block body
        let (body_statements, body_standalone_comments) = match default {
            YulSwitchOptions::Default(_, block) => {
                collect_yul_statements(&block.statements, &internal_comments, source)
            }
            _ => (vec![], vec![]), // Shouldn't happen
        };

        collected_cases.push(CommentedYulCase {
            case: default.clone(),
            leading_comments,
            trailing_comments,
            body_statements,
            body_standalone_comments,
        });
    }

    collected_cases
}

/// Collect a YUL block with all its statements and comments
pub fn collect_yul_block(block: &YulBlock, comments: &[Comment], source: &str) -> CollectedYulBlock {
    let (statements, standalone_comments) =
        collect_yul_statements(&block.statements, comments, source);

    CollectedYulBlock {
        statements,
        standalone_comments,
    }
}