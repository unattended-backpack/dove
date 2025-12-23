//! IR builders for top-level Solidity elements

use super::ir::IRElement;
use crate::ir_builder::comments::*;
use crate::ir_builder::expressions::*;
use crate::ir_builder::text_with_word_breaks;
use crate::collector::model::*;
use solang_parser::pt::*;

/// Build structured IR for variable NatSpec comments, with type info for mapping TODOs
fn build_variable_natspec_ir_with_type(
    natspec_lines: &[String],
    ty: &Expression,
) -> Vec<IRElement> {
    let mut ir = vec![];

    // Check if this is a mapping - mappings require @custom:param and @custom:return
    let is_mapping = matches!(ty, Expression::Type(_, Type::Mapping { .. }));
    let param_tag = if is_mapping { "@custom:param" } else { "@param" };
    let return_tag = if is_mapping { "@custom:return" } else { "@return" };

    // Start comment block
    ir.push(IRElement::text("/**"));

    // Content lines - HardLineBreak at start ensures first line gets indented
    let mut content_lines = vec![IRElement::HardLineBreak];

    // Collect notice/description and separate tag types
    let mut description_lines = Vec::new();
    let mut param_tags = Vec::new();
    let mut return_tags = Vec::new();
    let mut other_tags = Vec::new();
    // Track current section for multi-line continuation
    let mut in_description = false;

    for line in natspec_lines {
        let trimmed = line.trim();

        if trimmed.starts_with("@notice ") {
            let notice_text = trimmed.strip_prefix("@notice ").unwrap_or("").trim();
            description_lines.push(notice_text.to_string());
            in_description = true;
        } else if trimmed.starts_with("@dev ") {
            // @dev content is appended to description
            let dev_text = trimmed.strip_prefix("@dev ").unwrap_or("").trim();
            description_lines.push(dev_text.to_string());
            in_description = true;
        } else if trimmed.starts_with("@param ") || trimmed.starts_with("@custom:param ") {
            in_description = false;
            let rest = trimmed
                .strip_prefix("@param ")
                .or_else(|| trimmed.strip_prefix("@custom:param "))
                .unwrap();
            if let Some((name, desc)) = rest.split_once(' ') {
                // Don't prefix "TODO" placeholder with underscore
                let prefixed_name = if name.starts_with('_') || name == "TODO" {
                    name.to_string()
                } else {
                    format!("_{}", name)
                };
                param_tags.push(format!("{} {} {}", param_tag, prefixed_name, desc));
            } else {
                // Don't prefix "TODO" placeholder with underscore
                let prefixed_name = if rest.starts_with('_') || rest == "TODO" {
                    rest.to_string()
                } else {
                    format!("_{}", rest)
                };
                param_tags.push(format!("{} {}", param_tag, prefixed_name));
            }
        } else if trimmed.starts_with("@return ") || trimmed.starts_with("@custom:return ") {
            in_description = false;
            let rest = trimmed
                .strip_prefix("@return ")
                .or_else(|| trimmed.strip_prefix("@custom:return "))
                .unwrap();
            if let Some((name, desc)) = rest.split_once(' ') {
                // Don't prefix "TODO" placeholder with underscore
                let prefixed_name = if name.starts_with('_') || name == "TODO" {
                    name.to_string()
                } else {
                    format!("_{}", name)
                };
                return_tags.push(format!("{} {} {}", return_tag, prefixed_name, desc));
            } else {
                // Don't prefix "TODO" placeholder with underscore
                let prefixed_name = if rest.starts_with('_') || rest == "TODO" {
                    rest.to_string()
                } else {
                    format!("_{}", rest)
                };
                return_tags.push(format!("{} {}", return_tag, prefixed_name));
            }
        } else if trimmed.starts_with("@") {
            in_description = false;
            other_tags.push(trimmed.to_string());
        } else if !trimmed.is_empty() {
            // Continuation line - append to description if we were in description section
            if in_description || !description_lines.is_empty() {
                if !description_lines.is_empty() {
                    let last = description_lines.last_mut().unwrap();
                    last.push(' ');
                    last.push_str(trimmed);
                } else {
                    description_lines.push(trimmed.to_string());
                }
            }
        }
    }

    // Count mapping parameters needed and add TODOs for missing ones
    let (needed_params, needed_returns) = count_mapping_params(ty);

    // For mappings: if we have extra @param and 0 @return, convert to @return FIRST
    if needed_returns > 0 && param_tags.len() > needed_params && return_tags.is_empty() {
        while param_tags.len() > needed_params && return_tags.len() < needed_returns {
            let extra = param_tags.pop().unwrap();
            let as_return = extra.replacen(param_tag, return_tag, 1);
            return_tags.push(as_return);
        }
    }

    // Add TODO param tags for missing ones
    while param_tags.len() < needed_params {
        param_tags.push(format!("{} TODO", param_tag));
    }

    // Add TODO return tags for missing ones
    while return_tags.len() < needed_returns {
        return_tags.push(format!("{} TODO", return_tag));
    }

    // Track if we've added any section (for blank line logic)
    let mut has_previous_section = false;

    // Add description
    if !description_lines.is_empty() {
        let combined = description_lines.join(" ");
        content_lines.push(IRElement::group(text_with_word_breaks(&combined)));
        has_previous_section = true;
    } else {
        content_lines.push(IRElement::text("TODO"));
        has_previous_section = true;
    }

    // Add @param tags with blank line before (only if we have params)
    if !param_tags.is_empty() {
        if has_previous_section {
            content_lines.push(IRElement::HardLineBreak);
            content_lines.push(IRElement::HardLineBreak);
        }
        for tag in &param_tags {
            content_lines.push(IRElement::text(tag));
            content_lines.push(IRElement::HardLineBreak);
        }
        content_lines.pop();
        has_previous_section = true;
    }

    // Add @return tags with blank line before
    if !return_tags.is_empty() {
        if has_previous_section && !param_tags.is_empty() {
            content_lines.push(IRElement::HardLineBreak);
            content_lines.push(IRElement::HardLineBreak);
        } else if has_previous_section {
            content_lines.push(IRElement::HardLineBreak);
            content_lines.push(IRElement::HardLineBreak);
        }
        for tag in &return_tags {
            content_lines.push(IRElement::text(tag));
            content_lines.push(IRElement::HardLineBreak);
        }
        content_lines.pop();
    }

    // Add other tags
    if !other_tags.is_empty() {
        content_lines.push(IRElement::HardLineBreak);
        for tag in &other_tags {
            content_lines.push(IRElement::text(tag));
            content_lines.push(IRElement::HardLineBreak);
        }
        content_lines.pop();
    }

    // Wrap content in indent
    ir.push(IRElement::indent(content_lines));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Count the number of @param and @return tags needed for a mapping type
fn count_mapping_params(ty: &Expression) -> (usize, usize) {
    match ty {
        Expression::Type(_, Type::Mapping { value, .. }) => {
            // Key needs 1 @param, then recurse for nested mappings
            let (inner_params, inner_returns) = count_mapping_params(value);
            (1 + inner_params, inner_returns.max(1)) // At least 1 return for the final value
        }
        _ => (0, 0), // Non-mapping types don't need param/return docs (only mappings do)
    }
}

/// Build IR for a pragma directive
pub fn build_pragma_ir(pragma: &CommentedElement<Box<PragmaDirective>>) -> Vec<IRElement> {
    with_comments_no_spdx(&pragma.leading_comments, &pragma.trailing_comments, || {
        let mut ir = vec![IRElement::text("pragma"), IRElement::text(" ")];

        match pragma.element.as_ref() {
            PragmaDirective::Identifier(_, ident, value) => {
                if let Some(id) = ident {
                    ir.push(IRElement::text(&id.name));
                }
                if let Some(val) = value {
                    ir.push(IRElement::text(" "));
                    ir.push(IRElement::text(&val.name));
                }
            }
            PragmaDirective::Version(_, _, versions) => {
                ir.push(IRElement::text("solidity"));

                for (i, version) in versions.iter().enumerate() {
                    if i > 0 {
                        ir.push(IRElement::text(" || "));
                    } else {
                        ir.push(IRElement::text(" "));
                    }

                    // Format version requirement
                    match version {
                        VersionComparator::Plain { version, .. } => {
                            ir.push(IRElement::text(&version.join(".")));
                        }
                        VersionComparator::Operator { op, version, .. } => {
                            match op {
                                VersionOp::Greater => ir.push(IRElement::text(">")),
                                VersionOp::GreaterEq => ir.push(IRElement::text(">=")),
                                VersionOp::Less => ir.push(IRElement::text("<")),
                                VersionOp::LessEq => ir.push(IRElement::text("<=")),
                                VersionOp::Caret => ir.push(IRElement::text("^")),
                                VersionOp::Tilde => ir.push(IRElement::text("~")),
                                VersionOp::Wildcard => ir.push(IRElement::text("*")),
                                VersionOp::Exact => {}
                            }
                            ir.push(IRElement::text(&version.join(".")));
                        }
                        VersionComparator::Or { .. } => {
                            // Should not appear at this level
                            ir.push(IRElement::text("/* or version */"));
                        }
                        VersionComparator::Range { from, to, .. } => {
                            ir.push(IRElement::text(&format!(
                                "{} - {}",
                                from.join("."),
                                to.join(".")
                            )));
                        }
                    }
                }
            }
            PragmaDirective::StringLiteral(_, ident, value) => {
                ir.push(IRElement::text(&ident.name));
                ir.push(IRElement::text(" "));
                ir.push(IRElement::text(&format!("\"{}\"", value.string)));
            }
        }

        ir.push(IRElement::text(";"));
        ir
    })
}

/// Extract contract/symbol name from import path
/// e.g., "@openzeppelin/contracts/access/Ownable.sol" -> "Ownable"
fn extract_symbol_from_path(path: &ImportPath) -> String {
    let path_str = match path {
        ImportPath::Filename(s) => &s.string,
        ImportPath::Path(p) => {
            return p
                .identifiers
                .last()
                .map(|i| i.name.clone())
                .unwrap_or_default()
        }
    };

    // Extract filename from path
    let filename = path_str.split('/').last().unwrap_or(path_str);

    // Remove .sol extension
    filename
        .strip_suffix(".sol")
        .unwrap_or(filename)
        .to_string()
}

/// Build IR for an import directive
/// Note: Comments are intentionally dropped since imports get reordered
pub fn build_import_ir(import: &CommentedElement<Box<Import>>) -> Vec<IRElement> {
    // Don't use with_comments - drop comments since imports are reordered
    let mut ir = vec![IRElement::text("import"), IRElement::text(" ")];

    match import.element.as_ref() {
        Import::Plain(path, _) => {
            // Convert plain import to named import: import "path" -> import { Name } from "path"
            let symbol = extract_symbol_from_path(path);
            ir.push(IRElement::text("{ "));
            ir.push(IRElement::text(&symbol));
            ir.push(IRElement::text(" }"));
            ir.push(IRElement::text(" from"));
            // Wrap the path in indent so it gets proper indentation when breaking
            ir.push(IRElement::indent(vec![
                IRElement::SoftLineBreak,
                format_import_path(path),
            ]));
        }
        Import::GlobalSymbol(path, symbol, _) => {
            // import "path" as Symbol
            ir.push(format_import_path(path));
            ir.push(IRElement::text(" as "));
            ir.push(IRElement::text(&symbol.name));
        }
        Import::Rename(path, imports, _) => {
            ir.push(IRElement::text("{ "));

            // Group import symbols for potential line wrapping
            let mut symbols_group = vec![];
            for (i, (from, to)) in imports.iter().enumerate() {
                if i > 0 {
                    symbols_group.push(IRElement::text(", "));
                }
                symbols_group.push(IRElement::text(&from.name));
                if let Some(to_ident) = to {
                    symbols_group.push(IRElement::text(" as "));
                    symbols_group.push(IRElement::text(&to_ident.name));
                }
            }

            ir.extend(symbols_group);
            ir.push(IRElement::text(" }"));
            ir.push(IRElement::text(" from"));
            // Wrap the path in indent so it gets proper indentation when breaking
            ir.push(IRElement::indent(vec![
                IRElement::SoftLineBreak,
                format_import_path(path),
            ]));
        }
    }

    ir.push(IRElement::text(";"));
    vec![IRElement::group(ir)]
}

/// Format an import path
fn format_import_path(path: &ImportPath) -> IRElement {
    match path {
        ImportPath::Filename(s) => IRElement::text(format!("\"{}\"", s.string)),
        ImportPath::Path(path) => format_identifier_path(path),
    }
}

/// Build IR for a using directive
pub fn build_using_directive_ir(using: &CommentedElement<Box<Using>>) -> Vec<IRElement> {
    with_comments(&using.leading_comments, &using.trailing_comments, || {
        let mut ir = vec![IRElement::text("using"), IRElement::text(" ")];

        // Format the library/functions list
        match &using.element.list {
            UsingList::Library(lib) => {
                ir.push(format_identifier_path(lib));
            }
            UsingList::Functions(funcs) => {
                ir.push(IRElement::text("{"));

                let mut funcs_group = vec![];
                for (i, func) in funcs.iter().enumerate() {
                    if i > 0 {
                        funcs_group.push(IRElement::text(","));
                        funcs_group.push(IRElement::SoftLineBreak);
                    }
                    funcs_group.push(format_identifier_path(&func.path));
                    if let Some(op) = &func.oper {
                        funcs_group.push(IRElement::text(" as "));
                        funcs_group.push(IRElement::text(match op {
                            UserDefinedOperator::BitwiseAnd => "&",
                            UserDefinedOperator::BitwiseNot => "~",
                            UserDefinedOperator::BitwiseOr => "|",
                            UserDefinedOperator::BitwiseXor => "^",
                            UserDefinedOperator::Add => "+",
                            UserDefinedOperator::Divide => "/",
                            UserDefinedOperator::Modulo => "%",
                            UserDefinedOperator::Multiply => "*",
                            UserDefinedOperator::Subtract => "-",
                            UserDefinedOperator::Negate => "-",
                            UserDefinedOperator::Equal => "==",
                            UserDefinedOperator::More => ">",
                            UserDefinedOperator::MoreEqual => ">=",
                            UserDefinedOperator::Less => "<",
                            UserDefinedOperator::LessEqual => "<=",
                            UserDefinedOperator::NotEqual => "!=",
                        }));
                    }
                }

                if funcs.len() > 2 {
                    ir.push(IRElement::indent(vec![
                        IRElement::SoftLineBreak,
                        IRElement::group(funcs_group),
                        IRElement::SoftLineBreak,
                    ]));
                } else {
                    ir.extend(funcs_group);
                }

                ir.push(IRElement::text("}"));
            }
            UsingList::Error => {
                ir.push(IRElement::text("ERROR"));
            }
        }

        ir.push(IRElement::text(" for "));

        // Format the target type
        match &using.element.ty {
            Some(ty) => {
                ir.push(format_expression(ty));
            }
            None => {
                ir.push(IRElement::text("*"));
            }
        }

        // Add 'global' if present
        if using.element.global.is_some() {
            ir.push(IRElement::text(" global"));
        }

        ir.push(IRElement::text(";"));
        vec![IRElement::group(ir)]
    })
}

/// Build IR for a type definition
pub fn build_type_definition_ir(
    type_def: &CommentedElement<Box<TypeDefinition>>,
) -> Vec<IRElement> {
    with_comments(
        &type_def.leading_comments,
        &type_def.trailing_comments,
        || {
            vec![
                IRElement::text("type"),
                IRElement::text(" "),
                IRElement::text(&type_def.element.name.name),
                IRElement::text(" is "),
                format_expression(&type_def.element.ty),
                IRElement::text(";"),
            ]
        },
    )
}

fn format_variable_attribute(attr: &VariableAttribute) -> IRElement {
    match attr {
        VariableAttribute::Visibility(vis) => format_visibility(vis),
        VariableAttribute::Constant(_) => IRElement::text("constant"),
        VariableAttribute::Immutable(_) => IRElement::text("immutable"),
        VariableAttribute::Override(_, _) => IRElement::text("override"),
    }
}

/// Build IR for a variable definition
pub fn build_variable_definition_ir(
    var_def: &CommentedElement<Box<VariableDefinition>>,
) -> Vec<IRElement> {
    let mut ir = vec![];

    // Check if variable has any documentation
    let has_docs = !var_def.leading_comments.is_empty() || !var_def.trailing_comments.is_empty();

    if has_docs {
        // Check if these are NatSpec doc comments that should be formatted as a block
        let mut natspec_lines = Vec::new();
        let mut is_natspec = false;
        let mut has_regular_comments = false;

        for comment in &var_def.leading_comments {
            match comment {
                Comment::DocLine(_, content) => {
                    let trimmed = content.trim_start_matches("///").trim();
                    if trimmed.starts_with("@") {
                        is_natspec = true;
                    }
                    natspec_lines.push(trimmed.to_string());
                }
                Comment::DocBlock(_, content) => {
                    if content.contains("@") {
                        // Already a NatSpec block - use build_natspec_comment_ir
                        ir.extend(build_natspec_comment_ir(content));
                        ir.push(IRElement::HardLineBreak);
                        continue;
                    } else {
                        ir.push(build_comment(comment));
                        ir.push(IRElement::HardLineBreak);
                    }
                }
                _ => {
                    has_regular_comments = true;
                    ir.push(build_comment(comment));
                    ir.push(IRElement::HardLineBreak);
                }
            }
        }

        // Append trailing comments to natspec_lines so they become part of the block comment
        for comment in &var_def.trailing_comments {
            match comment {
                Comment::Line(_, content) | Comment::DocLine(_, content) => {
                    let trimmed = content
                        .trim_start_matches("///")
                        .trim_start_matches("//")
                        .trim();
                    if !trimmed.is_empty() {
                        natspec_lines.push(trimmed.to_string());
                    }
                }
                Comment::Block(_, content) | Comment::DocBlock(_, content) => {
                    let trimmed = content
                        .trim_start_matches("/*")
                        .trim_end_matches("*/")
                        .trim();
                    if !trimmed.is_empty() {
                        natspec_lines.push(trimmed.to_string());
                    }
                }
            }
        }

        // If we collected NatSpec doc lines, determine format based on complexity
        if is_natspec && !natspec_lines.is_empty() {
            // Check if this is a simple description-only comment (no @param/@return/etc tags)
            // @notice and @dev are considered description tags, not requiring block format
            let has_other_tags = natspec_lines.iter().any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("@")
                    && !trimmed.starts_with("@notice")
                    && !trimmed.starts_with("@dev")
            });

            // Check if the variable is a mapping (needs block format for param/return docs)
            let is_mapping = matches!(
                &var_def.element.ty,
                Expression::Type(_, Type::Mapping { .. })
            );

            if has_other_tags || is_mapping {
                // Complex NatSpec with @param, @return, etc. OR mapping - use block format
                ir.extend(build_variable_natspec_ir_with_type(
                    &natspec_lines,
                    &var_def.element.ty,
                ));
                ir.push(IRElement::HardLineBreak);
            } else {
                // Simple description - use single line format with @notice/@dev stripped
                let description: String = natspec_lines
                    .iter()
                    .map(|line| {
                        let trimmed = line.trim();
                        trimmed
                            .strip_prefix("@notice ")
                            .or_else(|| trimmed.strip_prefix("@dev "))
                            .unwrap_or(trimmed)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                ir.push(IRElement::comment(description, true));
                ir.push(IRElement::HardLineBreak);
            }
        } else if !has_regular_comments && !natspec_lines.is_empty() {
            // Doc comments without NatSpec tags - just output as-is
            for comment in &var_def.leading_comments {
                ir.push(build_comment(comment));
                ir.push(IRElement::HardLineBreak);
            }
        }
    } else {
        // Generate TODO documentation for undocumented state variables
        // Mappings need full NatSpec with @custom:param and @custom:return
        let is_mapping = matches!(
            &var_def.element.ty,
            Expression::Type(_, Type::Mapping { .. })
        );
        if is_mapping {
            ir.extend(build_variable_natspec_ir_with_type(&[], &var_def.element.ty));
            ir.push(IRElement::HardLineBreak);
        } else {
            ir.push(IRElement::text("/// TODO"));
            ir.push(IRElement::HardLineBreak);
        }
    }

    // Build type and variable name
    ir.push(format_expression(&var_def.element.ty));

    // Add attributes (visibility, mutability, etc.)
    for attr in &var_def.element.attrs {
        ir.push(IRElement::text(" "));
        ir.push(format_variable_attribute(attr));
    }

    // Add variable name
    ir.push(IRElement::text(" "));
    if let Some(name) = &var_def.element.name {
        ir.push(IRElement::text(&name.name));
    }

    // Add initializer if present
    if let Some(init_expr) = &var_def.element.initializer {
        // Wrap in a group so long assignments can break after the =
        // When it fits: bytes32 constant X = 0x123...;
        // When too long: bytes32 constant X =
        //                  0x123...;
        ir.push(IRElement::group(vec![
            IRElement::text(" ="),
            IRElement::indent(vec![
                IRElement::SoftLineBreak,
                format_expression(init_expr),
                IRElement::text(";"),
            ]),
        ]));
    } else {
        ir.push(IRElement::text(";"));
    }

    // Handle type comments if present (these would be within complex types like mappings)
    if let Some(type_comments) = &var_def.type_comments {
        // For now, we'll add any comments found in type expressions as inline comments
        // This is a simplified implementation - a full implementation would need to
        // integrate these comments into the type expression formatting
        if type_comments.has_comments() {
            // Add a comment indicating there are type-level comments that need handling
            ir.push(IRElement::text(" /* type has internal comments */"));
        }
    }

    // Trailing comments are now incorporated into the leading natspec block above

    ir
}
