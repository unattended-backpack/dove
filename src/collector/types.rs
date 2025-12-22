//! Type-related collection functions (structs, enums, events, errors)

use super::model::*;
use crate::collector::comments::collect_element_comments;
use crate::collector::utils::get_loc_range;
use solang_parser::helpers::CodeLocation;
use solang_parser::pt::*;

/// Collect struct fields with their associated comments
pub fn collect_struct_fields(
    fields: &[VariableDeclaration],
    comments: &[Comment],
    source: &str,
) -> Vec<CommentedElement<Box<VariableDeclaration>>> {
    let mut collected_fields = Vec::new();
    let mut remaining_comments = comments.to_vec();

    for field in fields {
        let (start, end) = get_loc_range(&field.loc);
        let (leading_comments, trailing_comments) =
            collect_element_comments(start, end, &mut remaining_comments, source);

        // Collect type expression comments if present
        let type_comments = collect_type_expression_comments(&field.ty, comments, source);

        collected_fields.push(CommentedElement {
            element: Box::new(field.clone()),
            leading_comments,
            trailing_comments,
            type_comments,
        });
    }

    collected_fields
}

/// Collect enum values with their associated comments
pub fn collect_enum_values(
    values: &[Option<Identifier>],
    comments: &[Comment],
    source: &str,
) -> Vec<CommentedElement<Option<Identifier>>> {
    let mut collected_values = Vec::new();
    let mut remaining_comments = comments.to_vec();

    for value in values {
        if let Some(ident) = value {
            let (start, end) = get_loc_range(&ident.loc);
            let (leading_comments, trailing_comments) =
                collect_element_comments(start, end, &mut remaining_comments, source);

            collected_values.push(CommentedElement {
                element: value.clone(),
                leading_comments,
                trailing_comments,
                type_comments: None,
            });
        } else {
            // None case - no identifier, no comments
            collected_values.push(CommentedElement {
                element: None,
                leading_comments: vec![],
                trailing_comments: vec![],
                type_comments: None,
            });
        }
    }

    collected_values
}

/// Collect event parameters with their associated comments
pub fn collect_event_parameters(
    params: &[EventParameter],
    comments: &[Comment],
    source: &str,
) -> Vec<CommentedElement<Box<EventParameter>>> {
    let mut collected_params = Vec::new();
    let mut remaining_comments = comments.to_vec();

    for param in params {
        let (start, end) = get_loc_range(&param.loc);
        let (leading_comments, trailing_comments) =
            collect_element_comments(start, end, &mut remaining_comments, source);

        collected_params.push(CommentedElement {
            element: Box::new(param.clone()),
            leading_comments,
            trailing_comments,
            type_comments: None,
        });
    }

    collected_params
}

/// Collect error parameters with their associated comments
pub fn collect_error_parameters(
    params: &[ErrorParameter],
    comments: &[Comment],
    source: &str,
) -> Vec<CommentedElement<Box<ErrorParameter>>> {
    let mut collected_params = Vec::new();
    let mut remaining_comments = comments.to_vec();

    for param in params {
        let (start, end) = get_loc_range(&param.loc);
        let (leading_comments, trailing_comments) =
            collect_element_comments(start, end, &mut remaining_comments, source);

        collected_params.push(CommentedElement {
            element: Box::new(param.clone()),
            leading_comments,
            trailing_comments,
            type_comments: None,
        });
    }

    collected_params
}

/// Collect type comment start positions for tracking
pub fn collect_type_comment_starts(
    type_comments: &TypeExpressionComments,
    comment_starts: &mut std::collections::HashSet<usize>,
) {
    use crate::collector::utils::get_loc_start;

    for comment in &type_comments.key_leading_comments {
        comment_starts.insert(get_loc_start(&comment.loc()));
    }
    for comment in &type_comments.key_trailing_comments {
        comment_starts.insert(get_loc_start(&comment.loc()));
    }
    for comment in &type_comments.value_leading_comments {
        comment_starts.insert(get_loc_start(&comment.loc()));
    }
    for comment in &type_comments.value_trailing_comments {
        comment_starts.insert(get_loc_start(&comment.loc()));
    }
    for comment in &type_comments.element_leading_comments {
        comment_starts.insert(get_loc_start(&comment.loc()));
    }
    for comment in &type_comments.element_trailing_comments {
        comment_starts.insert(get_loc_start(&comment.loc()));
    }
    // Recursively collect from nested type comments
    if let Some(nested) = &type_comments.nested_type_comments {
        collect_type_comment_starts(nested, comment_starts);
    }
    // Collect from function parameter/return comments
    for param_comments in &type_comments.function_param_comments {
        for comment in &param_comments.leading {
            comment_starts.insert(get_loc_start(&comment.loc()));
        }
        for comment in &param_comments.trailing {
            comment_starts.insert(get_loc_start(&comment.loc()));
        }
        if let Some(nested_type) = &param_comments.type_comments {
            collect_type_comment_starts(nested_type, comment_starts);
        }
    }
    for return_comments in &type_comments.function_return_comments {
        for comment in &return_comments.leading {
            comment_starts.insert(get_loc_start(&comment.loc()));
        }
        for comment in &return_comments.trailing {
            comment_starts.insert(get_loc_start(&comment.loc()));
        }
        if let Some(nested_type) = &return_comments.type_comments {
            collect_type_comment_starts(nested_type, comment_starts);
        }
    }
}

/// Get bounds of an expression  
pub fn get_expression_bounds(expr: &Expression) -> (usize, usize) {
    use crate::collector::utils::{get_loc_end, get_loc_start};

    match expr {
        Expression::Type(loc, _) => (get_loc_start(loc), get_loc_end(loc)),
        Expression::Variable(ident) => (get_loc_start(&ident.loc), get_loc_end(&ident.loc)),
        Expression::ArraySubscript(loc, _, _) => (get_loc_start(loc), get_loc_end(loc)),
        _ => (0, 0),
    }
}

/// Find the position of the arrow operator (=>) in a mapping type expression
fn find_arrow_position(source: &str, start: usize, end: usize) -> Option<usize> {
    find_token_position(source, "=>", start, end)
}

/// Find token position in source
fn find_token_position(source: &str, token: &str, start: usize, end: usize) -> Option<usize> {
    if start >= end || end > source.len() {
        return None;
    }

    let search_area = &source[start..end];
    search_area.find(token).map(|pos| start + pos)
}

/// Collect comments within type expressions
pub fn collect_type_expression_comments(
    ty: &Expression,
    comments: &[Comment],
    source: &str,
) -> Option<Box<TypeExpressionComments>> {
    use crate::collector::utils::{get_loc_end, get_loc_range, get_loc_start};

    match ty {
        Expression::Type(loc, type_expr) => {
            match type_expr {
                Type::Mapping {
                    loc: mapping_loc,
                    key,
                    key_name: _,
                    value,
                    value_name: _,
                } => {
                    let mut type_comments = TypeExpressionComments::new();
                    let (mapping_start, mapping_end) = get_loc_range(mapping_loc);

                    // Find the arrow position
                    if let Some(arrow_pos) = find_arrow_position(source, mapping_start, mapping_end)
                    {
                        // Get key expression bounds
                        let (key_start, key_end) = get_expression_bounds(key);

                        // Get value expression bounds
                        let (value_start, value_end) = get_expression_bounds(value);

                        // Collect comments before the key
                        for comment in comments {
                            let comment_start = get_loc_start(&comment.loc());
                            let comment_end = get_loc_end(&comment.loc());

                            // Leading comments before key
                            if comment_end <= key_start && comment_start > mapping_start {
                                type_comments.key_leading_comments.push(comment.clone());
                            }
                            // Trailing comments after key (including same line after =>)
                            else if comment_start >= key_end && comment_start < value_start {
                                // Check if comment is on same line as arrow
                                let arrow_line_end = source[arrow_pos..]
                                    .find('\n')
                                    .map(|i| arrow_pos + i)
                                    .unwrap_or(source.len());
                                if comment_start <= arrow_line_end {
                                    // Same line as arrow - treat as key trailing
                                    type_comments.key_trailing_comments.push(comment.clone());
                                } else {
                                    // Different line - treat as value leading
                                    type_comments.value_leading_comments.push(comment.clone());
                                }
                            }
                            // Trailing comments after value
                            else if comment_start >= value_end && comment_end < mapping_end {
                                type_comments.value_trailing_comments.push(comment.clone());
                            }
                        }

                        // Recursively collect comments for nested value type
                        if let Expression::Type(_, Type::Mapping { .. }) = &**value {
                            type_comments.nested_type_comments =
                                collect_type_expression_comments(value, comments, source);
                        }
                    }

                    if type_comments.has_comments() {
                        Some(Box::new(type_comments))
                    } else {
                        None
                    }
                }
                _ => None, // Other type expressions not supported yet
            }
        }
        _ => None, // Not a type expression
    }
}
