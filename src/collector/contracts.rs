//! Contract-related collection functions

use crate::collector::comments::{
    find_trailing_comments, remove_comments_by_position,
    separate_internal_and_after_comments, take_last_comment_group_before,
};
use crate::collector::functions::collect_function_signature_comments;
use crate::collector::statements::collect_statements;
use crate::collector::types::{
    collect_enum_values, collect_error_parameters, collect_event_parameters,
    collect_struct_fields, collect_type_comment_starts, collect_type_expression_comments,
};
use crate::collector::utils::{get_loc_end, get_loc_range, get_loc_start};
use super::model::*;
use solang_parser::pt::*;
use solang_parser::helpers::CodeLocation;

/// Get location range for a contract part
fn get_contract_part_location(part: &ContractPart) -> (usize, usize) {
    match part {
        ContractPart::StructDefinition(s) => get_loc_range(&s.loc()),
        ContractPart::EventDefinition(e) => get_loc_range(&e.loc()),
        ContractPart::EnumDefinition(e) => get_loc_range(&e.loc()),
        ContractPart::ErrorDefinition(e) => get_loc_range(&e.loc()),
        ContractPart::VariableDefinition(v) => get_loc_range(&v.loc()),
        ContractPart::FunctionDefinition(f) => get_loc_range(&f.loc()),
        ContractPart::TypeDefinition(t) => get_loc_range(&t.loc()),
        ContractPart::Annotation(a) => get_loc_range(&a.loc()),
        ContractPart::Using(u) => get_loc_range(&u.loc()),
        ContractPart::StraySemicolon(loc) => get_loc_range(loc),
    }
}

/// Collect contract parts with their associated comments
pub fn collect_contract_parts(
    parts: &[ContractPart],
    comments: &[Comment],
    source: &str,
) -> CollectedContractElements {
    let mut collected = CollectedContractElements::new();
    let mut remaining_comments = comments.to_vec();

    // Process each contract part in order
    for part in parts {
        let (start, end) = get_contract_part_location(part);

        // Take the last comment group before this element as its leading comments
        let (leading_comments, rest) =
            take_last_comment_group_before(&remaining_comments, source, start);
        remaining_comments = rest;

        // Find trailing comments on the same line (they could be after the element)
        let trailing_comments = find_trailing_comments(&remaining_comments, source, end);

        // Separate comments into: internal, trailing, and everything else
        let (internal_comments, after_element, _) = separate_internal_and_after_comments(
            remaining_comments,
            &trailing_comments,
            start,
            end,
        );

        // Update remaining comments (everything after this element, excluding trailing)
        remaining_comments = after_element;

        match part {
            ContractPart::Using(using_directive) => {
                collected.using_directives.push(CommentedElement {
                    element: using_directive.clone(),
                    leading_comments,
                    trailing_comments,
                    type_comments: None,
                });
            }
            ContractPart::TypeDefinition(type_def) => {
                collected.types.push(CommentedElement {
                    element: type_def.clone(),
                    leading_comments,
                    trailing_comments,
                    type_comments: None,
                });
            }
            ContractPart::EnumDefinition(enum_def) => {
                // Process enum with its value comments
                let values = collect_enum_values(&enum_def.values, &internal_comments, source);

                collected.enums.push(CollectedEnum {
                    definition: CommentedElement {
                        element: enum_def.clone(),
                        leading_comments,
                        trailing_comments,
                        type_comments: None,
                    },
                    values,
                });
            }
            ContractPart::StructDefinition(struct_def) => {
                // Process struct with its field comments
                let fields = collect_struct_fields(&struct_def.fields, &internal_comments, source);

                collected.structs.push(CollectedStruct {
                    definition: CommentedElement {
                        element: struct_def.clone(),
                        leading_comments,
                        trailing_comments,
                        type_comments: None,
                    },
                    fields,
                });
            }
            ContractPart::ErrorDefinition(error) => {
                // Process error with its parameter comments
                let parameters =
                    collect_error_parameters(&error.fields, &internal_comments, source);

                collected.errors.push(CollectedError {
                    definition: CommentedElement {
                        element: error.clone(),
                        leading_comments,
                        trailing_comments,
                        type_comments: None,
                    },
                    parameters,
                });
            }
            ContractPart::EventDefinition(event) => {
                // Process event with its parameter comments
                let parameters =
                    collect_event_parameters(&event.fields, &internal_comments, source);

                collected.events.push(CollectedEvent {
                    definition: CommentedElement {
                        element: event.clone(),
                        leading_comments,
                        trailing_comments,
                        type_comments: None,
                    },
                    parameters,
                });
            }
            ContractPart::VariableDefinition(variable) => {
                // Collect type expression comments if present
                let type_comments =
                    collect_type_expression_comments(&variable.ty, &internal_comments, source);

                collected.variables.push(CommentedElement {
                    element: variable.clone(),
                    leading_comments,
                    trailing_comments,
                    type_comments,
                });
            }
            ContractPart::FunctionDefinition(function) => {
                // Collect signature comments for parameters and returns
                let signature_comments = collect_function_signature_comments(
                    &function.params,
                    &function.returns,
                    &internal_comments,
                    source,
                );

                // Filter out signature comments from internal comments before processing body
                let mut body_internal_comments = internal_comments.clone();

                // First, filter out ALL comments that are within parameter bounds
                if !function.params.is_empty() {
                    let first_param_start = get_loc_start(&function.params[0].0);
                    let last_param_end = get_loc_end(&function.params[function.params.len() - 1].0);
                    body_internal_comments.retain(|c| {
                        let c_start = get_loc_start(&c.loc());
                        c_start < first_param_start || c_start > last_param_end
                    });
                }

                // Also filter out comments within return parameter bounds
                if !function.returns.is_empty() {
                    let first_return_start = get_loc_start(&function.returns[0].0);
                    let last_return_end =
                        get_loc_end(&function.returns[function.returns.len() - 1].0);
                    body_internal_comments.retain(|c| {
                        let c_start = get_loc_start(&c.loc());
                        c_start < first_return_start || c_start > last_return_end
                    });
                }

                if let Some(sig_comments) = &signature_comments {
                    // Collect all signature comment locations
                    let mut sig_comment_starts = std::collections::HashSet::new();
                    for param_comments in &sig_comments.parameters {
                        for comment in &param_comments.leading {
                            sig_comment_starts.insert(get_loc_start(&comment.loc()));
                        }
                        for comment in &param_comments.trailing {
                            sig_comment_starts.insert(get_loc_start(&comment.loc()));
                        }
                        // Also collect type expression comments
                        if let Some(type_comments) = &param_comments.type_comments {
                            collect_type_comment_starts(type_comments, &mut sig_comment_starts);
                        }
                    }
                    for return_comments in &sig_comments.returns {
                        for comment in &return_comments.leading {
                            sig_comment_starts.insert(get_loc_start(&comment.loc()));
                        }
                        for comment in &return_comments.trailing {
                            sig_comment_starts.insert(get_loc_start(&comment.loc()));
                        }
                        // Also collect type expression comments
                        if let Some(type_comments) = &return_comments.type_comments {
                            collect_type_comment_starts(type_comments, &mut sig_comment_starts);
                        }
                    }
                    // Remove signature comments from body internal comments
                    let sig_positions: Vec<usize> = sig_comment_starts.into_iter().collect();
                    remove_comments_by_position(&mut body_internal_comments, &sig_positions);
                }

                // Process function body statements with their comments
                let (body_statements, body_standalone_comments) = if let Some(body) = &function.body
                {
                    collect_statements(&[body.clone()], &body_internal_comments, source)
                } else {
                    (vec![], vec![])
                };

                collected.functions.push(CollectedFunction {
                    definition: CommentedElement {
                        element: function.clone(),
                        leading_comments,
                        trailing_comments,
                        type_comments: None,
                    },
                    body_statements,
                    body_standalone_comments,
                    signature_comments,
                });
            }
            ContractPart::Annotation(_) => {
                // Annotations are non-standard solang-parser extensions, not needed
            }
            ContractPart::StraySemicolon(_) => {
                // Stray semicolons are never output in formatted code
            }
        }
    }

    // Any remaining comments are standalone within the contract
    collected.standalone_comments = remaining_comments;

    collected
}