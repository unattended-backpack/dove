//! Function-related collection utilities

use crate::collector::comments::{
    find_trailing_comments, remove_comments_by_position, take_last_comment_group_before,
};
use crate::collector::types::{collect_type_expression_comments, get_expression_bounds};
use crate::collector::utils::{get_loc_end, get_loc_range, get_loc_start};
use super::model::*;
use solang_parser::pt::*;
use solang_parser::helpers::CodeLocation;

/// Collect function signature comments (parameters and returns)
pub fn collect_function_signature_comments(
    params: &[(Loc, Option<Parameter>)],
    returns: &[(Loc, Option<Parameter>)],
    comments: &[Comment],
    source: &str,
) -> Option<Box<FunctionSignatureComments>> {
    let mut remaining_comments = comments.to_vec();
    let mut param_comments = Vec::new();
    let mut return_comments = Vec::new();

    // Find the closing parenthesis after parameters
    let params_closing_paren_pos = if !params.is_empty() {
        let last_param_end = get_loc_end(&params[params.len() - 1].0);
        // Search for the closing parenthesis after the last parameter
        source[last_param_end..]
            .find(')')
            .map(|offset| last_param_end + offset)
    } else {
        None
    };

    // Collect parameter comments
    for (i, (loc, param_opt)) in params.iter().enumerate() {
        let (start, end) = get_loc_range(loc);

        // Take the last comment group before this parameter
        let (mut leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // If parameter exists, filter out comments that are within the type expression
        if let Some(param) = param_opt {
            let (type_start, type_end) = get_expression_bounds(&param.ty);
            leading_comments.retain(|c| {
                let c_start = get_loc_start(&c.loc());
                // Keep comment only if it's not within the type expression
                c_start < type_start || c_start >= type_end
            });
        }

        // Find trailing comments on the same line
        let mut trailing_comments = find_trailing_comments(&remaining_comments, source, end);

        // For the last parameter, only include comments before the closing parenthesis
        if i == params.len() - 1 && params_closing_paren_pos.is_some() {
            let closing_paren = params_closing_paren_pos.unwrap();
            trailing_comments.retain(|c| get_loc_start(&c.loc()) < closing_paren);
        }

        // Collect type expression comments if parameter exists
        let type_comments = if let Some(param) = param_opt {
            collect_type_expression_comments(&param.ty, comments, source)
        } else {
            None
        };

        // Remove trailing comments from remaining
        let trailing_positions: Vec<usize> = trailing_comments
            .iter()
            .map(|c| get_loc_start(&c.loc()))
            .collect();
        remove_comments_by_position(&mut remaining_comments, &trailing_positions);

        param_comments.push(FunctionParameterComments {
            leading: leading_comments,
            trailing: trailing_comments,
            type_comments,
        });
    }

    // Find the closing parenthesis after returns
    let returns_closing_paren_pos = if !returns.is_empty() {
        let last_return_end = get_loc_end(&returns[returns.len() - 1].0);
        // Search for the closing parenthesis after the last return
        source[last_return_end..]
            .find(')')
            .map(|offset| last_return_end + offset)
    } else {
        None
    };

    // Collect return comments
    for (i, (loc, param_opt)) in returns.iter().enumerate() {
        let (start, end) = get_loc_range(loc);

        // Take the last comment group before this return
        let (mut leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // If parameter exists, filter out comments that are within the type expression
        if let Some(param) = param_opt {
            let (type_start, type_end) = get_expression_bounds(&param.ty);
            leading_comments.retain(|c| {
                let c_start = get_loc_start(&c.loc());
                // Keep comment only if it's not within the type expression
                c_start < type_start || c_start >= type_end
            });
        }

        // Find trailing comments on the same line
        let mut trailing_comments = find_trailing_comments(&remaining_comments, source, end);

        // For the last return parameter, only include comments before the closing parenthesis
        if i == returns.len() - 1 && returns_closing_paren_pos.is_some() {
            let closing_paren = returns_closing_paren_pos.unwrap();
            trailing_comments.retain(|c| get_loc_start(&c.loc()) < closing_paren);
        }

        // Collect type expression comments if return parameter exists
        let type_comments = if let Some(param) = param_opt {
            collect_type_expression_comments(&param.ty, comments, source)
        } else {
            None
        };

        // Remove trailing comments from remaining
        let trailing_positions: Vec<usize> = trailing_comments
            .iter()
            .map(|c| get_loc_start(&c.loc()))
            .collect();
        remove_comments_by_position(&mut remaining_comments, &trailing_positions);

        return_comments.push(FunctionParameterComments {
            leading: leading_comments,
            trailing: trailing_comments,
            type_comments,
        });
    }

    // Only return Some if we actually have comments
    if param_comments
        .iter()
        .any(|pc| !pc.leading.is_empty() || !pc.trailing.is_empty())
        || return_comments
            .iter()
            .any(|rc| !rc.leading.is_empty() || !rc.trailing.is_empty())
    {
        Some(Box::new(FunctionSignatureComments {
            parameters: param_comments,
            returns: return_comments,
        }))
    } else {
        None
    }
}