//! Statement collection logic

use crate::collector::comments::{
    find_trailing_comments, separate_internal_and_after_comments, take_last_comment_group_before,
};
use crate::collector::utils::{get_loc_end, get_loc_range, get_loc_start};
use crate::collector::yul::collect_yul_block;
use super::model::*;
use solang_parser::pt::*;
use solang_parser::helpers::CodeLocation;

/// Get location range for a statement
fn get_statement_location(stmt: &Statement) -> (usize, usize) {
    match stmt {
        Statement::Block { loc, .. } => get_loc_range(loc),
        Statement::Assembly { loc, .. } => get_loc_range(loc),
        Statement::Args(loc, _) => get_loc_range(loc),
        Statement::If(loc, _, _, _) => get_loc_range(loc),
        Statement::While(loc, _, _) => get_loc_range(loc),
        Statement::Expression(loc, _) => get_loc_range(loc),
        Statement::VariableDefinition(loc, _, _) => get_loc_range(loc),
        Statement::For(loc, _, _, _, _) => get_loc_range(loc),
        Statement::DoWhile(loc, _, _) => get_loc_range(loc),
        Statement::Continue(loc) => get_loc_range(loc),
        Statement::Break(loc) => get_loc_range(loc),
        Statement::Return(loc, _) => get_loc_range(loc),
        Statement::Revert(loc, _, _) => get_loc_range(loc),
        Statement::RevertNamedArgs(loc, _, _) => get_loc_range(loc),
        Statement::Emit(loc, _) => get_loc_range(loc),
        Statement::Try(loc, _, _, _) => get_loc_range(loc),
        Statement::Error(loc) => get_loc_range(loc),
    }
}

/// Process a block or statement recursively
fn process_block_or_statement(
    stmt: &Statement,
    internal_comments: &[Comment],
    source: &str,
) -> (Vec<CommentedStatement>, Vec<Comment>) {
    match stmt {
        Statement::Block { statements, .. } => {
            collect_statements(statements, internal_comments, source)
        }
        _ => (vec![], vec![]),
    }
}

/// Get location range for a catch clause
fn get_catch_clause_location(clause: &CatchClause) -> (usize, usize) {
    match clause {
        CatchClause::Simple(loc, _, _) => get_loc_range(loc),
        CatchClause::Named(loc, _, _, _) => get_loc_range(loc),
    }
}

/// Collect catch clauses with their associated comments
pub fn collect_catch_clauses(
    clauses: &[CatchClause],
    comments: &[Comment],
    source: &str,
) -> Vec<CommentedCatchClause> {
    let mut collected_clauses = Vec::new();
    let mut remaining_comments = comments.to_vec();

    for clause in clauses {
        let (start, end) = get_catch_clause_location(clause);

        // Take the last comment group before this catch clause
        let (leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // Find trailing comments on the same line after the catch clause header
        let header_end = match clause {
            CatchClause::Simple(_, _, body) => get_statement_location(body).0,
            CatchClause::Named(_, _, _, body) => get_statement_location(body).0,
        };

        let trailing_comments = find_trailing_comments(&remaining_comments, source, header_end);

        // Separate internal comments and comments after the catch clause
        let (internal_comments, after_clause, _) =
            separate_internal_and_after_comments(remaining_comments, &trailing_comments, start, end);

        // Update remaining comments
        remaining_comments = after_clause;

        // Process the body of the catch clause
        let (body_statements, body_standalone_comments) = match clause {
            CatchClause::Simple(_, _, body) | CatchClause::Named(_, _, _, body) => {
                process_block_or_statement(body, &internal_comments, source)
            }
        };

        collected_clauses.push(CommentedCatchClause {
            clause: clause.clone(),
            leading_comments,
            trailing_comments,
            body_statements,
            body_standalone_comments,
        });
    }

    collected_clauses
}

/// Collect statements with their associated comments
pub fn collect_statements(
    statements: &[Statement],
    comments: &[Comment],
    source: &str,
) -> (Vec<CommentedStatement>, Vec<Comment>) {
    let mut collected_statements = Vec::new();
    let mut remaining_comments = comments.to_vec();

    for stmt in statements {
        let (start, end) = get_statement_location(stmt);

        // Take the last comment group before this statement
        let (leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // Find trailing comments on the same line
        let mut trailing_comments = find_trailing_comments(&remaining_comments, source, end);

        // Special handling for if statements - also look for inline comments after the condition
        if let Statement::If(_loc, cond, _, _) = stmt {
            let cond_end = get_loc_end(&cond.loc());
            let inline_after_cond = find_trailing_comments(&remaining_comments, source, cond_end);
            // Add inline comments that aren't already in trailing_comments
            for comment in inline_after_cond {
                if !trailing_comments
                    .iter()
                    .any(|c| get_loc_start(&c.loc()) == get_loc_start(&comment.loc()))
                {
                    trailing_comments.push(comment);
                }
            }
        }

        // Special handling for while statements - also look for inline comments after the condition
        if let Statement::While(_loc, cond, _) = stmt {
            let cond_end = get_loc_end(&cond.loc());
            let inline_after_cond = find_trailing_comments(&remaining_comments, source, cond_end);
            // Add inline comments that aren't already in trailing_comments
            for comment in inline_after_cond {
                if !trailing_comments
                    .iter()
                    .any(|c| get_loc_start(&c.loc()) == get_loc_start(&comment.loc()))
                {
                    trailing_comments.push(comment);
                }
            }
        }

        // Special handling for try statements - look for comments after the returns clause
        if let Statement::Try(_loc, _expr, returns_and_body, _) = stmt {
            if let Some((params, success_body)) = returns_and_body {
                // Find where to look for trailing comments
                // If there are parameters, look after the last parameter
                // Otherwise, look before the success body
                let search_pos = if let Some((last_param_loc, _)) = params.last() {
                    get_loc_end(last_param_loc)
                } else {
                    // No parameters, look before the success body
                    get_statement_location(success_body).0
                };

                let inline_after_returns =
                    find_trailing_comments(&remaining_comments, source, search_pos);
                // Add inline comments that aren't already in trailing_comments
                for comment in inline_after_returns {
                    if !trailing_comments
                        .iter()
                        .any(|c| get_loc_start(&c.loc()) == get_loc_start(&comment.loc()))
                    {
                        trailing_comments.push(comment);
                    }
                }
            }
        }

        // Separate comments into: internal, trailing, and everything else
        let (internal_comments, after_statement, _) = separate_internal_and_after_comments(
            remaining_comments,
            &trailing_comments,
            start,
            end,
        );

        // Update remaining comments (everything after this statement, excluding trailing)
        remaining_comments = after_statement;

        // Handle nested statements recursively
        let (
            processed_stmt,
            nested_statements,
            nested_standalone,
            yul_block,
            catch_clauses,
            else_branch,
            for_component_comments,
        ) = match stmt {
            Statement::Block { statements, .. } => {
                // Recursively process block statements
                let (nested, standalone) =
                    collect_statements(statements, &internal_comments, source);
                (
                    stmt.clone(),
                    Some(nested),
                    Some(standalone),
                    None,
                    None,
                    None,
                    None,
                )
            }
            Statement::If(_loc, _cond, then_stmt, else_stmt) => {
                // Need to partition internal comments for then vs else branches
                let then_start = get_statement_location(then_stmt).0;
                let then_end = get_statement_location(then_stmt).1;

                // Comments for the then branch are those within the then statement
                let then_internal_comments: Vec<Comment> = internal_comments
                    .iter()
                    .filter(|c| {
                        let c_start = get_loc_start(&c.loc());
                        c_start >= then_start && c_start < then_end
                    })
                    .cloned()
                    .collect();

                // Process the then branch if it's a block
                let (then_nested, then_standalone) = 
                    process_block_or_statement(then_stmt, &then_internal_comments, source);

                // Process the else branch if it exists
                let else_branch = if let Some(else_stmt) = else_stmt {
                    // Find where the else keyword would be (between then_stmt end and else_stmt start)
                    let then_end = get_statement_location(then_stmt).1;
                    let else_start = get_statement_location(else_stmt).0;

                    // Look for comments between the closing brace of then and the else statement
                    let mut else_leading = Vec::new();
                    for comment in &internal_comments {
                        let comment_start = get_loc_start(&comment.loc());
                        if comment_start >= then_end && comment_start < else_start {
                            else_leading.push(comment.clone());
                        }
                    }

                    // Find trailing comments for else branch
                    let mut else_trailing = Vec::new();

                    // For else-if, look for comments after the condition closing paren
                    if let Statement::If(_loc, cond, _, _) = &**else_stmt {
                        // Find the end of the condition (closing paren)
                        let cond_end = get_loc_end(&cond.loc());

                        // Find the end of the line containing the condition
                        let line_end = source[cond_end..]
                            .find('\n')
                            .map(|i| cond_end + i)
                            .unwrap_or(source.len());

                        // Find comments between condition end and end of line
                        for comment in &internal_comments {
                            let comment_start = get_loc_start(&comment.loc());
                            if comment_start > cond_end && comment_start < line_end {
                                else_trailing.push(comment.clone());
                            }
                        }
                    } else {
                        // For regular else block, look for comments on the same line as "else"
                        // Find the line containing the else keyword
                        let else_line_end = source[else_start..]
                            .find('\n')
                            .map(|i| else_start + i)
                            .unwrap_or(source.len());

                        for comment in &internal_comments {
                            let comment_start = get_loc_start(&comment.loc());
                            // Check if comment is after else_start and on the same line
                            if comment_start >= else_start && comment_start < else_line_end {
                                else_trailing.push(comment.clone());
                            }
                        }
                    }

                    // Filter comments for else branch - only those within the else statement
                    let else_stmt_start = get_statement_location(else_stmt).0;
                    let else_stmt_end = get_statement_location(else_stmt).1;

                    // Also filter out the else trailing comments
                    let else_trailing_starts: std::collections::HashSet<usize> = else_trailing
                        .iter()
                        .map(|c| get_loc_start(&c.loc()))
                        .collect();

                    let else_internal_comments: Vec<Comment> = internal_comments
                        .iter()
                        .filter(|c| {
                            let c_start = get_loc_start(&c.loc());
                            c_start >= else_stmt_start
                                && c_start < else_stmt_end
                                && !else_trailing_starts.contains(&c_start)
                        })
                        .cloned()
                        .collect();

                    // Process else statement based on its type
                    let (else_nested, else_standalone) = match &**else_stmt {
                        Statement::Block { statements, .. } => {
                            collect_statements(statements, &else_internal_comments, source)
                        }
                        Statement::If(_, _, _, _) => {
                            // For else-if, we need to collect it as a single statement
                            collect_statements(
                                &[(**else_stmt).clone()],
                                &else_internal_comments,
                                source,
                            )
                        }
                        _ => (vec![], vec![]),
                    };

                    Some(Box::new(CommentedElseBranch {
                        leading_comments: else_leading,
                        trailing_comments: else_trailing,
                        statement: (**else_stmt).clone(),
                        nested_statements: if else_nested.is_empty() {
                            None
                        } else {
                            Some(else_nested)
                        },
                        nested_standalone_comments: if else_standalone.is_empty() {
                            None
                        } else {
                            Some(else_standalone)
                        },
                    }))
                } else {
                    None
                };

                // Collect any remaining internal comments that aren't in then or else as standalone
                let mut all_standalone = then_standalone;

                // Keep track of which comments were already assigned to else leading
                let else_leading_starts: std::collections::HashSet<usize> = if else_branch.is_some()
                {
                    else_branch
                        .as_ref()
                        .unwrap()
                        .leading_comments
                        .iter()
                        .map(|c| get_loc_start(&c.loc()))
                        .collect()
                } else {
                    std::collections::HashSet::new()
                };

                // Add comments that are between branches or after all branches
                for comment in &internal_comments {
                    let c_start = get_loc_start(&comment.loc());
                    let in_then = c_start >= then_start && c_start < then_end;
                    let in_else = if let Some(else_stmt) = else_stmt {
                        let else_start = get_statement_location(else_stmt).0;
                        let else_end = get_statement_location(else_stmt).1;
                        c_start >= else_start && c_start < else_end
                    } else {
                        false
                    };

                    // If comment is not in then or else branch, and not already in else leading, it's a standalone comment
                    if !in_then && !in_else && !else_leading_starts.contains(&c_start) {
                        all_standalone.push(comment.clone());
                    }
                }

                (
                    stmt.clone(),
                    Some(then_nested),
                    Some(all_standalone),
                    None,
                    None,
                    else_branch,
                    None,
                )
            }
            Statement::While(_loc, _cond, body) => {
                // Process the body if it's a block
                let (nested, standalone) = match &**body {
                    Statement::Block { statements, .. } => {
                        collect_statements(statements, &internal_comments, source)
                    }
                    _ => (vec![], vec![]),
                };
                (
                    stmt.clone(),
                    Some(nested),
                    Some(standalone),
                    None,
                    None,
                    None,
                    None,
                )
            }
            Statement::For(_loc, init, cond, update, body) => {
                // For loops have complex internal structure with init, condition, and update
                // We need to partition comments between these components and the body

                // First, determine the bounds of each component
                let body_start = if let Some(body_stmt) = body.as_ref() {
                    get_statement_location(body_stmt).0
                } else {
                    end // If no body, use the for statement end
                };

                // Separate comments: those before the body go to the for components,
                // those within the body go to the body
                let mut body_comments = Vec::new();
                let mut for_header_comments = Vec::new();

                for comment in &internal_comments {
                    let comment_start = get_loc_start(&comment.loc());
                    if comment_start >= body_start {
                        body_comments.push(comment.clone());
                    } else {
                        // These are comments within the for(...) part
                        for_header_comments.push(comment.clone());
                    }
                }

                // Process the body if it exists
                let (nested, standalone) = if let Some(body_stmt) = body.as_ref() {
                    process_block_or_statement(body_stmt, &body_comments, source)
                } else {
                    (vec![], vec![])
                };

                // Now collect the for loop component comments
                let for_component_comments = if !for_header_comments.is_empty() {
                    let mut component_comments = ForLoopComponentComments {
                        init_leading: Vec::new(),
                        init_trailing: Vec::new(),
                        condition_leading: Vec::new(),
                        condition_trailing: Vec::new(),
                        update_leading: Vec::new(),
                        update_trailing: Vec::new(),
                    };

                    // Get bounds for each component
                    let init_bounds = init.as_ref().and_then(|s| {
                        let (start, end) = get_statement_location(&**s);
                        Some((start, end))
                    });

                    let cond_bounds = cond.as_ref().map(|e| {
                        let loc = e.loc();
                        (get_loc_start(&loc), get_loc_end(&loc))
                    });

                    let update_bounds = update.as_ref().map(|e| {
                        let loc = e.loc();
                        (get_loc_start(&loc), get_loc_end(&loc))
                    });

                    // Sort comments by position
                    let mut sorted_comments = for_header_comments.clone();
                    sorted_comments.sort_by_key(|c| get_loc_start(&c.loc()));

                    // Assign comments to the appropriate component
                    for comment in sorted_comments {
                        let comment_start = get_loc_start(&comment.loc());

                        // Check if it's related to init
                        if let Some((init_start, init_end)) = init_bounds {
                            if comment_start < init_start {
                                component_comments.init_leading.push(comment.clone());
                                continue;
                            } else if comment_start >= init_end
                                && !source[init_end..comment_start].contains('\n')
                            {
                                // Check if it's a trailing comment after init (on the same line)
                                component_comments.init_trailing.push(comment.clone());
                                continue;
                            }
                        }

                        // Check if it's related to condition
                        if let Some((cond_start, cond_end)) = cond_bounds {
                            if comment_start < cond_start
                                && (init_bounds.is_none() || comment_start > init_bounds.unwrap().1)
                            {
                                component_comments.condition_leading.push(comment.clone());
                                continue;
                            } else if comment_start >= cond_end
                                && !source[cond_end..comment_start].contains('\n')
                            {
                                // Check if it's a trailing comment after condition (on the same line)
                                component_comments.condition_trailing.push(comment.clone());
                                continue;
                            }
                        }

                        // Check if it's related to update
                        if let Some((update_start, update_end)) = update_bounds {
                            if comment_start < update_start
                                && (cond_bounds.is_none() || comment_start > cond_bounds.unwrap().1)
                            {
                                component_comments.update_leading.push(comment.clone());
                                continue;
                            } else if comment_start >= update_end
                                && !source[update_end..comment_start].contains('\n')
                            {
                                // Check if it's a trailing comment after update (on the same line)
                                component_comments.update_trailing.push(comment.clone());
                                continue;
                            }
                        }
                    }

                    Some(Box::new(component_comments))
                } else {
                    None
                };

                (
                    stmt.clone(),
                    Some(nested),
                    Some(standalone),
                    None,
                    None,
                    None,
                    for_component_comments,
                )
            }
            Statement::DoWhile(_loc, body, _cond) => {
                // Process the body if it's a block
                let (nested, standalone) = process_block_or_statement(body, &internal_comments, source);
                (
                    stmt.clone(),
                    Some(nested),
                    Some(standalone),
                    None,
                    None,
                    None,
                    None,
                )
            }
            Statement::Try(_loc, _expr, returns_and_body, catch_clauses) => {
                // First, we need to separate comments between the try body and catch clauses
                let mut try_body_comments = Vec::new();
                let mut catch_comments = Vec::new();

                if let Some((_, success_body)) = returns_and_body {
                    let (body_start, body_end) = get_statement_location(success_body);

                    // Partition comments between try body and catch clauses
                    for comment in &internal_comments {
                        let comment_start = get_loc_start(&comment.loc());
                        if comment_start >= body_start && comment_start < body_end {
                            try_body_comments.push(comment.clone());
                        } else {
                            catch_comments.push(comment.clone());
                        }
                    }
                } else {
                    // No try body, all comments go to catch clauses
                    catch_comments = internal_comments.clone();
                }

                // Process the success body if present
                let (nested, standalone) = if let Some((_, success_body)) = returns_and_body {
                    process_block_or_statement(success_body, &try_body_comments, source)
                } else {
                    (vec![], vec![])
                };

                // Process all catch clauses with their comments
                let commented_catch_clauses =
                    collect_catch_clauses(catch_clauses, &catch_comments, source);
                (
                    stmt.clone(),
                    Some(nested),
                    Some(standalone),
                    None,
                    Some(commented_catch_clauses),
                    None,
                    None,
                )
            }
            Statement::Assembly {
                loc: _,
                dialect: _,
                flags: _,
                block,
            } => {
                // Assembly blocks now have proper YUL comment collection
                let yul_block = collect_yul_block(block, &internal_comments, source);
                (stmt.clone(), None, None, Some(yul_block), None, None, None)
            }
            _ => (stmt.clone(), None, None, None, None, None, None), // For other statements, no nested processing needed
        };

        collected_statements.push(CommentedStatement {
            statement: processed_stmt,
            leading_comments,
            trailing_comments,
            nested_statements,
            nested_standalone_comments: nested_standalone,
            yul_block,
            catch_clauses,
            else_branch,
            for_component_comments,
        });
    }

    // Return collected statements and any remaining comments as standalone
    (collected_statements, remaining_comments)
}