//! Type definition builders for the IR

use crate::ir_builder::comments::*;
use crate::ir_builder::expressions::*;
use crate::ir_builder::text_with_word_breaks;
use super::ir::IRElement;
use crate::collector::model::*;
use solang_parser::pt::*;
use std::collections::HashMap;

/// Parse struct comments handling both @param and @custom:field tags
fn parse_struct_comments_custom(comments: &[Comment]) -> (String, HashMap<String, String>) {
    let mut description_parts = Vec::new();
    let mut field_docs = HashMap::new();
    let mut in_custom_field = false;
    let mut current_field = String::new();
    let mut current_field_doc = Vec::new();

    for comment in comments {
        let comment_text = match comment {
            Comment::Line(_, text) => text,
            Comment::Block(_, text) => text,
            Comment::DocLine(_, text) => text,
            Comment::DocBlock(_, text) => text,
        };

        // Extract content without comment markers
        let content = if comment_text.trim().starts_with("///") {
            comment_text.trim().strip_prefix("///").unwrap_or("").trim()
        } else if comment_text.trim().starts_with("//") {
            comment_text.trim().strip_prefix("//").unwrap_or("").trim()
        } else if comment_text.trim().starts_with("/**") && comment_text.trim().ends_with("*/") {
            comment_text
                .trim()
                .strip_prefix("/**")
                .unwrap_or("")
                .strip_suffix("*/")
                .unwrap_or("")
                .trim()
        } else {
            comment_text.trim()
        };

        // Process each line, tracking indentation for continuation detection
        // Track context: which tag type we're in and the current field name for @param
        #[derive(PartialEq, Clone)]
        enum Context {
            Description,
            Param(String), // field name
            CustomField(String), // field name
            Other,
        }
        let mut context = Context::Description;
        let mut base_indent: Option<usize> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            let leading_spaces = line.len() - line.trim_start().len();

            // Detect if this is a continuation line (more indented than base)
            let is_continuation = if let Some(base) = base_indent {
                leading_spaces > base && !trimmed.starts_with("@")
            } else {
                false
            };

            if trimmed.starts_with("@param ") {
                // Save any in-progress custom field
                if in_custom_field && !current_field.is_empty() {
                    field_docs.insert(
                        current_field.clone(),
                        current_field_doc.join(" ").trim().to_string(),
                    );
                    in_custom_field = false;
                    current_field.clear();
                    current_field_doc.clear();
                }

                if let Some(rest) = trimmed.strip_prefix("@param ") {
                    if let Some((field_name, desc)) = rest.split_once(' ') {
                        field_docs.insert(field_name.to_string(), desc.trim().to_string());
                        context = Context::Param(field_name.to_string());
                    } else {
                        // @param with no description yet
                        context = Context::Param(rest.to_string());
                    }
                }
                base_indent = Some(leading_spaces);
            } else if trimmed.starts_with("@custom:field ") {
                // Save any in-progress custom field
                if in_custom_field && !current_field.is_empty() {
                    field_docs.insert(
                        current_field.clone(),
                        current_field_doc.join(" ").trim().to_string(),
                    );
                }

                if let Some(rest) = trimmed.strip_prefix("@custom:field ") {
                    if let Some((field_name, desc)) = rest.split_once(' ') {
                        in_custom_field = true;
                        current_field = field_name.to_string();
                        current_field_doc = vec![desc.trim().to_string()];
                        context = Context::CustomField(field_name.to_string());
                    }
                }
                base_indent = Some(leading_spaces);
            } else if trimmed.starts_with("@notice ") {
                // Save any in-progress custom field
                if in_custom_field && !current_field.is_empty() {
                    field_docs.insert(
                        current_field.clone(),
                        current_field_doc.join(" ").trim().to_string(),
                    );
                    in_custom_field = false;
                    current_field.clear();
                    current_field_doc.clear();
                }

                description_parts
                    .push(trimmed.strip_prefix("@notice ").unwrap().trim().to_string());
                context = Context::Description;
                base_indent = Some(leading_spaces);
            } else if trimmed.starts_with("@dev ") {
                // Save any in-progress custom field
                if in_custom_field && !current_field.is_empty() {
                    field_docs.insert(
                        current_field.clone(),
                        current_field_doc.join(" ").trim().to_string(),
                    );
                    in_custom_field = false;
                    current_field.clear();
                    current_field_doc.clear();
                }

                description_parts.push(trimmed.strip_prefix("@dev ").unwrap().trim().to_string());
                context = Context::Description;
                base_indent = Some(leading_spaces);
            } else if trimmed.starts_with("@") {
                // Other tag, end custom field if any
                if in_custom_field && !current_field.is_empty() {
                    field_docs.insert(
                        current_field.clone(),
                        current_field_doc.join(" ").trim().to_string(),
                    );
                    in_custom_field = false;
                    current_field.clear();
                    current_field_doc.clear();
                }
                context = Context::Other;
                base_indent = Some(leading_spaces);
            } else if !trimmed.is_empty() {
                // Non-tag line - check if it's a continuation based on indentation
                if is_continuation {
                    // Continuation of previous tag
                    match &context {
                        Context::Param(field_name) => {
                            // Append to existing field doc
                            if let Some(existing) = field_docs.get_mut(field_name) {
                                existing.push(' ');
                                existing.push_str(trimmed);
                            }
                        }
                        Context::CustomField(_) if in_custom_field => {
                            current_field_doc.push(trimmed.to_string());
                        }
                        Context::Description => {
                            description_parts.push(trimmed.to_string());
                        }
                        _ => {
                            // Fallback to description
                            description_parts.push(trimmed.to_string());
                        }
                    }
                } else if in_custom_field {
                    // Non-indented line while in custom field - still continuation
                    current_field_doc.push(trimmed.to_string());
                } else {
                    // Regular description text at base indentation
                    description_parts.push(trimmed.to_string());
                    context = Context::Description;
                }
            }
        }
    }

    // Save any final in-progress custom field
    if in_custom_field && !current_field.is_empty() {
        field_docs.insert(
            current_field,
            current_field_doc.join(" ").trim().to_string(),
        );
    }

    let description = if description_parts.is_empty() {
        "TODO".to_string()
    } else {
        description_parts.join(" ")
    };

    (description, field_docs)
}

/// Format NatSpec for structs using structured IR elements
/// `inline_field_docs` contains documentation extracted from comments on struct fields
fn format_struct_natspec_ir(
    all_comments: &[Comment],
    _struct_name: &str,
    struct_fields: &[StructField],
    inline_field_docs: &HashMap<String, String>,
) -> Vec<IRElement> {
    use crate::ir_builder::text_with_word_breaks;

    let mut ir = vec![];

    // Parse comments with custom handling (from struct-level comments)
    let (description, mut field_docs) = parse_struct_comments_custom(&all_comments);

    // Merge inline field docs - inline docs take precedence if struct-level has none
    // If both exist, combine them (struct-level @param first, then inline)
    for (field_name, inline_doc) in inline_field_docs {
        if let Some(existing) = field_docs.get_mut(field_name) {
            // Combine: existing @param doc + inline comment
            existing.push(' ');
            existing.push_str(inline_doc);
        } else {
            // No struct-level doc, use inline
            field_docs.insert(field_name.clone(), inline_doc.clone());
        }
    }

    ir.push(IRElement::text("/**"));

    // Description section - HardLineBreak at start ensures first line gets indented
    let mut content = vec![IRElement::HardLineBreak];
    if description != "TODO" && !description.is_empty() {
        content.push(IRElement::group(text_with_word_breaks(&description)));
    } else {
        content.push(IRElement::text("TODO"));
    }

    // Add blank line before params if we have any
    if !struct_fields.is_empty() {
        content.push(IRElement::HardLineBreak);
        content.push(IRElement::HardLineBreak);

        // Add field documentation
        for field in struct_fields {
            if let Some(field_doc) = field_docs.get(&field.name) {
                // Handle potentially long field documentation
                // Wrap continuation in Indent for extra indentation on line wrap
                let mut continuation = vec![IRElement::SoftLineBreak];
                continuation.extend(text_with_word_breaks(field_doc));

                let param_group = vec![
                    IRElement::text(&format!("@param {}", field.name)),
                    IRElement::indent(continuation),
                ];

                content.push(IRElement::group(param_group));
            } else {
                content.push(IRElement::text(&format!("@param {} TODO", field.name)));
            }
            content.push(IRElement::HardLineBreak);
        }
        // Remove last HardLineBreak
        content.pop();
    }

    ir.push(IRElement::indent(content));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Convert an Expression (type) to a string representation
fn expression_to_type_string(expr: &Expression) -> String {
    match expr {
        Expression::Type(_, ty) => match ty {
            Type::Bool => "bool".to_string(),
            Type::Address => "address".to_string(),
            Type::AddressPayable => "address payable".to_string(),
            Type::Payable => "payable".to_string(),
            Type::Bytes(n) => format!("bytes{}", n),
            Type::Int(n) => format!("int{}", n),
            Type::Uint(n) => format!("uint{}", n),
            Type::String => "string".to_string(),
            Type::DynamicBytes => "bytes".to_string(),
            Type::Rational => "rational".to_string(),
            Type::Mapping { .. } => "mapping".to_string(), // TODO: handle mapping types properly
            Type::Function { .. } => "function".to_string(), // TODO: handle function types properly
        },
        Expression::Variable(ident) => ident.name.clone(),
        Expression::MemberAccess(_, expr, ident) => {
            format!("{}.{}", expression_to_type_string(expr), ident.name)
        }
        Expression::ArraySubscript(_, expr, _) => {
            format!("{}[]", expression_to_type_string(expr))
        }
        _ => "unknown".to_string(), // TODO: handle other expression types
    }
}

/// Build IR for an enum definition
pub fn build_enum_ir(enum_def: &CollectedEnum) -> Vec<IRElement> {
    let mut ir = vec![];

    // Check if enum has any documentation
    let has_docs = !enum_def.definition.leading_comments.is_empty()
        || !enum_def.definition.trailing_comments.is_empty();

    if has_docs {
        // Build structured NatSpec from existing comments
        ir.extend(build_enum_natspec_from_comments(enum_def));
        ir.push(IRElement::HardLineBreak);
    } else {
        // Generate NatSpec documentation for undocumented enums
        ir.extend(generate_enum_natspec(enum_def));
        ir.push(IRElement::HardLineBreak);
    }

    // Build the enum definition
    let enum_ir = build_enum_definition(enum_def);
    ir.extend(enum_ir);

    // Move trailing comments to the next line as leading comments
    if !enum_def.definition.trailing_comments.is_empty() {
        ir.push(IRElement::HardLineBreak);
        for comment in &enum_def.definition.trailing_comments {
            ir.push(build_comment(comment));
            ir.push(IRElement::HardLineBreak);
        }
    }

    ir
}

fn build_enum_definition(enum_def: &CollectedEnum) -> Vec<IRElement> {
    let mut ir = vec![
        IRElement::text("enum"),
        IRElement::text(" "),
        IRElement::text(&enum_def.definition.element.name.as_ref().unwrap().name),
        IRElement::text(" "),
        IRElement::text("{"),
    ];

    // Format enum values
    if !enum_def.values.is_empty() {
        let mut values_ir = vec![];

        for (i, value) in enum_def.values.iter().enumerate() {
            // Add leading comments for this value
            for comment in &value.leading_comments {
                values_ir.push(IRElement::HardLineBreak);
                values_ir.push(IRElement::indent(vec![build_comment(comment)]));
            }

            values_ir.push(IRElement::HardLineBreak);
            values_ir.push(IRElement::indent(vec![
                if let Some(name) = &value.element {
                    IRElement::text(&name.name)
                } else {
                    IRElement::text("")
                },
            ]));

            // Add trailing comma except for last element
            if i < enum_def.values.len() - 1 {
                values_ir.push(IRElement::text(","));
            }

            // Move trailing comments to the next line as leading comments
            if !value.trailing_comments.is_empty() {
                values_ir.push(IRElement::HardLineBreak);
                for comment in &value.trailing_comments {
                    values_ir.push(IRElement::indent(vec![build_comment(comment)]));
                    values_ir.push(IRElement::HardLineBreak);
                }
            }
        }

        ir.extend(values_ir);
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text("}"));
    ir
}

/// Build structured NatSpec from existing enum comments
/// Converts @dev/@notice docs into block format with @param for each value
/// Preserves existing @param documentation
fn build_enum_natspec_from_comments(enum_def: &CollectedEnum) -> Vec<IRElement> {
    let comments = &enum_def.definition.leading_comments;

    // Extract description and existing @param docs from comments
    let mut description_parts = Vec::new();
    let mut param_docs: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut current_param: Option<String> = None;

    for comment in comments {
        let content = match comment {
            Comment::DocLine(_, text) => text.trim_start_matches("///").trim().to_string(),
            Comment::DocBlock(_, text) => {
                // Strip block comment markers
                text.trim()
                    .trim_start_matches("/**")
                    .trim_end_matches("*/")
                    .trim()
                    .to_string()
            }
            Comment::Line(_, text) => text.trim_start_matches("//").trim().to_string(),
            Comment::Block(_, text) => text
                .trim()
                .trim_start_matches("/*")
                .trim_end_matches("*/")
                .trim()
                .to_string(),
        };

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("@dev ") {
                current_param = None;
                description_parts.push(
                    trimmed
                        .strip_prefix("@dev ")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
            } else if trimmed.starts_with("@notice ") {
                current_param = None;
                description_parts.push(
                    trimmed
                        .strip_prefix("@notice ")
                        .unwrap_or("")
                        .trim()
                        .to_string(),
                );
            } else if trimmed.starts_with("@param ") {
                // Parse @param name description
                if let Some(rest) = trimmed.strip_prefix("@param ") {
                    let mut parts = rest.splitn(2, char::is_whitespace);
                    if let Some(name) = parts.next() {
                        let desc = parts.next().unwrap_or("").trim().to_string();
                        param_docs.insert(name.to_string(), desc);
                        current_param = Some(name.to_string());
                    }
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with("@") {
                // Continuation line - append to current context
                if let Some(ref param_name) = current_param {
                    if let Some(existing) = param_docs.get_mut(param_name) {
                        existing.push(' ');
                        existing.push_str(trimmed);
                    }
                } else {
                    description_parts.push(trimmed.to_string());
                }
            } else if trimmed.starts_with("@") {
                // Other tag - reset current_param context
                current_param = None;
            }
        }
    }

    let description = if description_parts.is_empty() {
        "TODO".to_string()
    } else {
        description_parts.join(" ")
    };

    // Build block NatSpec
    let mut ir = vec![];
    ir.push(IRElement::text("/**"));

    let mut content = vec![IRElement::HardLineBreak];

    // Add description
    content.push(IRElement::group(text_with_word_breaks(&description)));

    // Add @param for each enum value - preserve existing docs or add TODO
    if !enum_def.values.is_empty() {
        content.push(IRElement::HardLineBreak);
        content.push(IRElement::HardLineBreak);

        for value in &enum_def.values {
            if let Some(value_name) = &value.element {
                if let Some(doc) = param_docs.get(&value_name.name) {
                    if doc.is_empty() {
                        content.push(IRElement::text(&format!("@param {} TODO", value_name.name)));
                    } else {
                        content.push(IRElement::text(&format!("@param {} {}", value_name.name, doc)));
                    }
                } else {
                    content.push(IRElement::text(&format!("@param {} TODO", value_name.name)));
                }
                content.push(IRElement::HardLineBreak);
            }
        }

        // Remove last HardLineBreak
        if let Some(IRElement::HardLineBreak) = content.last() {
            content.pop();
        }
    }

    ir.push(IRElement::indent(content));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Generate NatSpec documentation for an undocumented enum
fn generate_enum_natspec(enum_def: &CollectedEnum) -> Vec<IRElement> {
    let mut ir = vec![];

    ir.push(IRElement::text("/**"));

    // HardLineBreak at start ensures first line gets indented
    let mut content = vec![IRElement::HardLineBreak];

    // Add TODO description
    content.push(IRElement::text("TODO"));

    // Add documentation for enum values if any
    if !enum_def.values.is_empty() {
        content.push(IRElement::HardLineBreak);
        content.push(IRElement::HardLineBreak);

        for value in &enum_def.values {
            if let Some(value_name) = &value.element {
                content.push(IRElement::text(&format!("@param {} TODO", value_name.name)));
                content.push(IRElement::HardLineBreak);
            }
        }

        // Remove last HardLineBreak
        if let Some(IRElement::HardLineBreak) = content.last() {
            content.pop();
        }
    }

    ir.push(IRElement::indent(content));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Extract text content from a comment, stripping comment markers
fn extract_comment_text(comment: &Comment) -> String {
    let text = match comment {
        Comment::Line(_, t) | Comment::Block(_, t) | Comment::DocLine(_, t) | Comment::DocBlock(_, t) => t,
    };
    let trimmed = text.trim();
    // Strip comment markers
    if trimmed.starts_with("///") {
        trimmed.strip_prefix("///").unwrap_or("").trim().to_string()
    } else if trimmed.starts_with("//") {
        trimmed.strip_prefix("//").unwrap_or("").trim().to_string()
    } else if trimmed.starts_with("/*") && trimmed.ends_with("*/") {
        trimmed.strip_prefix("/*").unwrap_or("")
            .strip_suffix("*/").unwrap_or("")
            .trim()
            .lines()
            .map(|l| l.trim().strip_prefix("*").unwrap_or(l.trim()).trim())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        trimmed.to_string()
    }
}

/// Build IR for a struct definition
pub fn build_struct_ir(struct_def: &CollectedStruct) -> Vec<IRElement> {
    let mut ir = vec![];

    // Collect all struct comments
    let mut all_comments = Vec::new();
    all_comments.extend(struct_def.definition.leading_comments.clone());
    all_comments.extend(struct_def.definition.trailing_comments.clone());

    // Extract inline field comments (leading and trailing comments on struct fields)
    let mut inline_field_docs: HashMap<String, String> = HashMap::new();
    for field in &struct_def.fields {
        let field_name = field.element.name.as_ref().map(|n| n.name.clone()).unwrap_or_default();
        if field_name.is_empty() {
            continue;
        }

        let mut parts = Vec::new();

        // Leading comments on this field
        for comment in &field.leading_comments {
            let text = extract_comment_text(comment);
            if !text.is_empty() {
                parts.push(text);
            }
        }

        // Trailing comments on this field
        for comment in &field.trailing_comments {
            let text = extract_comment_text(comment);
            if !text.is_empty() {
                parts.push(text);
            }
        }

        if !parts.is_empty() {
            inline_field_docs.insert(field_name, parts.join(" "));
        }
    }

    // Extract field information for NatSpec
    let struct_fields: Vec<StructField> = struct_def
        .fields
        .iter()
        .map(|field| {
            let field_name = field
                .element
                .name
                .as_ref()
                .map(|n| n.name.clone())
                .unwrap_or_default();
            let field_type = expression_to_type_string(&field.element.ty);
            StructField {
                name: field_name,
                type_name: field_type,
            }
        })
        .collect();

    let struct_name = struct_def
        .definition
        .element
        .name
        .as_ref()
        .unwrap()
        .name
        .clone();

    // Generate or format NatSpec with structured IR, including inline field docs
    let natspec_ir = format_struct_natspec_ir(&all_comments, &struct_name, &struct_fields, &inline_field_docs);

    // Add the formatted NatSpec elements
    ir.extend(natspec_ir);
    ir.push(IRElement::HardLineBreak);

    // Build the struct definition
    ir.push(IRElement::text("struct"));
    ir.push(IRElement::text(" "));
    ir.push(IRElement::text(&struct_name));
    ir.push(IRElement::text(" "));
    ir.push(IRElement::text("{"));

    // Format struct fields
    if !struct_def.fields.is_empty() {
        let mut fields_ir = vec![];

        for field in &struct_def.fields {
            fields_ir.push(IRElement::HardLineBreak);
            fields_ir.push(IRElement::indent(vec![
                format_expression(&field.element.ty),
                IRElement::text(" "),
                if let Some(name) = &field.element.name {
                    IRElement::text(&name.name)
                } else {
                    IRElement::text("")
                },
                IRElement::text(";"),
            ]));
        }

        ir.extend(fields_ir);
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text("}"));
    ir
}

/// Build IR for an error definition
pub fn build_error_ir(error_def: &CollectedError) -> Vec<IRElement> {
    let mut ir = vec![];

    // Check if error has any documentation
    let has_docs = !error_def.definition.leading_comments.is_empty()
        || !error_def.definition.trailing_comments.is_empty();

    if has_docs {
        let comments = &error_def.definition.leading_comments;

        if !comments.is_empty() {
            if error_def.parameters.is_empty() {
                // No parameters - use simple comment format (let printer decide line/block)
                let combined_text = combine_error_comments(comments);
                ir.push(IRElement::comment(combined_text, true));
                ir.push(IRElement::HardLineBreak);
            } else {
                // Has parameters - build block NatSpec with @param TODOs
                ir.extend(build_error_natspec_from_comments(error_def, comments));
                ir.push(IRElement::HardLineBreak);
            }
        }
    } else {
        // Generate NatSpec documentation for undocumented errors
        ir.extend(generate_error_natspec(error_def));
        ir.push(IRElement::HardLineBreak);
    }

    // Build the error definition
    let error_ir = build_error_definition(error_def);
    ir.extend(error_ir);

    // Move trailing comments to the next line as leading comments
    if !error_def.definition.trailing_comments.is_empty() {
        ir.push(IRElement::HardLineBreak);
        for comment in &error_def.definition.trailing_comments {
            ir.push(build_comment(comment));
            ir.push(IRElement::HardLineBreak);
        }
    }

    ir
}

/// Build block NatSpec for an error from existing comments, adding @param TODOs
fn build_error_natspec_from_comments(
    error_def: &CollectedError,
    comments: &[Comment],
) -> Vec<IRElement> {
    let mut ir = vec![];

    // Extract description and existing @param docs from comments
    let (description, documented_params) = parse_error_comments(comments);

    ir.push(IRElement::text("/**"));

    let mut content = vec![IRElement::HardLineBreak];
    content.push(IRElement::group(text_with_word_breaks(&description)));

    // Add @param for parameters - only TODO if not already documented
    if !error_def.parameters.is_empty() {
        content.push(IRElement::HardLineBreak);
        content.push(IRElement::HardLineBreak);

        for param in &error_def.parameters {
            if let Some(param_name) = &param.element.name {
                if let Some(doc) = documented_params.get(&param_name.name) {
                    // Already documented - use existing doc
                    let mut continuation = vec![IRElement::SoftLineBreak];
                    continuation.extend(text_with_word_breaks(doc));
                    let param_group = vec![
                        IRElement::text(&format!("@param {}", param_name.name)),
                        IRElement::indent(continuation),
                    ];
                    content.push(IRElement::group(param_group));
                } else {
                    // Not documented - add TODO
                    content.push(IRElement::text(&format!("@param {} TODO", param_name.name)));
                }
                content.push(IRElement::HardLineBreak);
            }
        }

        // Remove last HardLineBreak
        if let Some(IRElement::HardLineBreak) = content.last() {
            content.pop();
        }
    }

    ir.push(IRElement::indent(content));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Parse error comments to extract description and existing @param entries
fn parse_error_comments(comments: &[Comment]) -> (String, HashMap<String, String>) {
    let mut description_parts = Vec::new();
    let mut param_docs = HashMap::new();
    let mut current_param: Option<String> = None;
    let mut current_param_doc = Vec::new();

    for comment in comments {
        let content = match comment {
            Comment::DocLine(_, text) => {
                text.trim().strip_prefix("///").unwrap_or(text.trim()).trim().to_string()
            }
            Comment::DocBlock(_, text) => {
                let inner = text.trim();
                let inner = inner.strip_prefix("/**").unwrap_or(inner);
                let inner = inner.strip_suffix("*/").unwrap_or(inner);
                inner.trim().to_string()
            }
            _ => String::new(),
        };

        // Track indentation for continuation detection
        let mut base_indent: Option<usize> = None;

        for line in content.lines() {
            let trimmed = line.trim();
            let leading_spaces = line.len() - line.trim_start().len();

            let is_continuation = if let Some(base) = base_indent {
                leading_spaces > base && !trimmed.starts_with("@")
            } else {
                false
            };

            if trimmed.starts_with("@param ") {
                // Save any in-progress param
                if let Some(param_name) = current_param.take() {
                    param_docs.insert(param_name, current_param_doc.join(" ").trim().to_string());
                    current_param_doc.clear();
                }

                if let Some(rest) = trimmed.strip_prefix("@param ") {
                    if let Some((name, doc)) = rest.split_once(' ') {
                        current_param = Some(name.to_string());
                        current_param_doc.push(doc.trim().to_string());
                    } else {
                        // @param name with no doc
                        current_param = Some(rest.to_string());
                    }
                }
                base_indent = Some(leading_spaces);
            } else if trimmed.starts_with("@notice ") {
                // Save any in-progress param
                if let Some(param_name) = current_param.take() {
                    param_docs.insert(param_name, current_param_doc.join(" ").trim().to_string());
                    current_param_doc.clear();
                }
                description_parts.push(trimmed.strip_prefix("@notice ").unwrap().trim().to_string());
                base_indent = Some(leading_spaces);
            } else if trimmed.starts_with("@") {
                // Other tag - save in-progress param
                if let Some(param_name) = current_param.take() {
                    param_docs.insert(param_name, current_param_doc.join(" ").trim().to_string());
                    current_param_doc.clear();
                }
                base_indent = Some(leading_spaces);
            } else if !trimmed.is_empty() {
                if is_continuation && current_param.is_some() {
                    // Continuation of param doc
                    current_param_doc.push(trimmed.to_string());
                } else if current_param.is_none() {
                    // Description text
                    description_parts.push(trimmed.to_string());
                }
            }
        }
    }

    // Save final param
    if let Some(param_name) = current_param {
        param_docs.insert(param_name, current_param_doc.join(" ").trim().to_string());
    }

    let description = if description_parts.is_empty() {
        "TODO".to_string()
    } else {
        description_parts.join(" ")
    };

    (description, param_docs)
}

/// Combine multiple error comments into a single string, stripping @notice
fn combine_error_comments(comments: &[Comment]) -> String {
    let mut parts = Vec::new();
    for comment in comments {
        let text = match comment {
            Comment::DocLine(_, content) => {
                let trimmed = content
                    .trim()
                    .strip_prefix("///")
                    .unwrap_or(content.trim())
                    .trim();
                trimmed
                    .strip_prefix("@notice ")
                    .unwrap_or(trimmed)
                    .to_string()
            }
            Comment::DocBlock(_, content) => {
                let text = content.trim().strip_prefix("/**").unwrap_or(content.trim());
                let text = text.strip_suffix("*/").unwrap_or(text).trim();
                text.strip_prefix("@notice ").unwrap_or(text).to_string()
            }
            _ => String::new(),
        };
        if !text.is_empty() {
            parts.push(text);
        }
    }
    parts.join(" ")
}

fn build_error_definition(error_def: &CollectedError) -> Vec<IRElement> {
    let mut ir = vec![
        IRElement::text("error"),
        IRElement::text(" "),
        IRElement::text(&error_def.definition.element.name.as_ref().unwrap().name),
        IRElement::text(" ("),
    ];

    // Format error parameters - always on separate lines when there are any
    if !error_def.parameters.is_empty() {
        let mut params_content = vec![IRElement::HardLineBreak];

        for (i, param) in error_def.parameters.iter().enumerate() {
            if i > 0 {
                params_content.push(IRElement::text(","));
                params_content.push(IRElement::HardLineBreak);
            }

            // Add type
            params_content.push(format_expression(&param.element.ty));

            // Add name if present
            if let Some(name) = &param.element.name {
                params_content.push(IRElement::text(" "));
                params_content.push(IRElement::text(&name.name));
            }
        }

        ir.push(IRElement::indent(params_content));
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text(");"));
    ir
}

/// Generate NatSpec documentation for an undocumented error
fn generate_error_natspec(error_def: &CollectedError) -> Vec<IRElement> {
    let mut ir = vec![];

    ir.push(IRElement::text("/**"));

    // HardLineBreak at start ensures first line gets indented
    let mut content = vec![IRElement::HardLineBreak];

    // Add TODO description
    content.push(IRElement::text("TODO"));

    // Add documentation for error parameters if any
    if !error_def.parameters.is_empty() {
        content.push(IRElement::HardLineBreak);
        content.push(IRElement::HardLineBreak);

        for param in &error_def.parameters {
            if let Some(param_name) = &param.element.name {
                content.push(IRElement::text(&format!("@param {} TODO", param_name.name)));
                content.push(IRElement::HardLineBreak);
            }
        }

        // Remove last HardLineBreak
        if let Some(IRElement::HardLineBreak) = content.last() {
            content.pop();
        }
    }

    ir.push(IRElement::indent(content));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}
