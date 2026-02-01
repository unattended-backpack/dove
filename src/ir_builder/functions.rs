//! Function definition builders for the IR

use super::ir::IRElement;
use crate::ir_builder::comments::*;
use crate::ir_builder::expressions::*;
use crate::ir_builder::statements::*;
use crate::ir_builder::text_with_word_breaks;
use crate::collector::model::*;
use solang_parser::pt::*;
use std::collections::{HashMap, HashSet};

/// Information about a named return variable that needs conversion
#[derive(Clone)]
struct ReturnVarInfo {
    /// Original name (e.g., "config_") - stored for potential future use
    #[allow(dead_code)]
    original_name: String,
    /// Transformed output name (e.g., "configOutput")
    output_name: String,
    /// The parameter definition (for type info)
    param: Parameter,
}

/// Check if we need a blank line before a statement in the function body
fn needs_blank_line_before_statement(statements: &[CommentedStatement], index: usize) -> bool {
    if index == 0 {
        return false;
    }

    let stmt = &statements[index];

    // Add blank line before statements with leading comments
    if !stmt.leading_comments.is_empty() {
        return true;
    }

    // Add blank line before nested if statements (when previous was also an if)
    match &stmt.statement {
        Statement::If(_, _, _, _) => {
            matches!(&statements[index - 1].statement, Statement::If(_, _, _, _))
        }
        _ => false,
    }
}

/// Build a map of return variable info from function returns
fn build_return_var_info(returns: &[(Loc, Option<Parameter>)]) -> HashMap<String, ReturnVarInfo> {
    let mut map = HashMap::new();
    for (_, ret) in returns {
        if let Some(param) = ret {
            if let Some(name) = &param.name {
                let original_name = name.name.clone();
                let output_name = transform_return_var_name(&original_name);
                map.insert(
                    original_name.clone(),
                    ReturnVarInfo {
                        original_name,
                        output_name,
                        param: param.clone(),
                    },
                );
            }
        }
    }
    map
}

/// Count how many times each return variable is used (appears) in the function body
/// This includes assignments, reads, and array accesses like `results[i]`
fn count_return_var_usages(
    statements: &[CommentedStatement],
    return_vars: &HashSet<String>,
) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for var in return_vars {
        counts.insert(var.clone(), 0);
    }
    count_usages_in_statements(statements, &mut counts);
    counts
}

fn count_usages_in_statements(statements: &[CommentedStatement], counts: &mut HashMap<String, usize>) {
    for stmt in statements {
        // Check the raw statement for usages
        count_usages_in_statement(&stmt.statement, counts);
        // Also check nested statements (e.g., for blocks, if statements)
        if let Some(nested) = &stmt.nested_statements {
            count_usages_in_statements(nested, counts);
        }
    }
}

fn count_usages_in_statement(stmt: &Statement, counts: &mut HashMap<String, usize>) {
    match stmt {
        Statement::Expression(_, expr) => {
            count_usages_in_expression(expr, counts);
        }
        Statement::VariableDefinition(_, _, init) => {
            if let Some(init_expr) = init {
                count_usages_in_expression(init_expr, counts);
            }
        }
        // Note: For Block, we rely on CommentedStatement::nested_statements being processed
        Statement::Block { .. } => {}
        Statement::If(_, cond, then_stmt, else_stmt) => {
            count_usages_in_expression(cond, counts);
            count_usages_in_statement(then_stmt, counts);
            if let Some(else_s) = else_stmt {
                count_usages_in_statement(else_s, counts);
            }
        }
        Statement::While(_, cond, body) => {
            count_usages_in_expression(cond, counts);
            count_usages_in_statement(body, counts);
        }
        Statement::DoWhile(_, body, cond) => {
            count_usages_in_statement(body, counts);
            count_usages_in_expression(cond, counts);
        }
        Statement::For(_, init, cond, update, body) => {
            if let Some(init_stmt) = init {
                count_usages_in_statement(init_stmt, counts);
            }
            if let Some(cond_expr) = cond {
                count_usages_in_expression(cond_expr, counts);
            }
            if let Some(update_expr) = update {
                count_usages_in_expression(update_expr, counts);
            }
            if let Some(body_stmt) = body {
                count_usages_in_statement(body_stmt, counts);
            }
        }
        Statement::Return(_, val) => {
            if let Some(val_expr) = val {
                count_usages_in_expression(val_expr, counts);
            }
        }
        _ => {}
    }
}

fn count_usages_in_expression(expr: &Expression, counts: &mut HashMap<String, usize>) {
    match expr {
        Expression::Variable(var) => {
            if let Some(count) = counts.get_mut(&var.name) {
                *count += 1;
            }
        }
        Expression::Assign(_, left, right) => {
            count_usages_in_expression(left, counts);
            count_usages_in_expression(right, counts);
        }
        Expression::ArraySubscript(_, array, index) => {
            count_usages_in_expression(array, counts);
            if let Some(idx) = index {
                count_usages_in_expression(idx, counts);
            }
        }
        Expression::MemberAccess(_, base, _) => {
            count_usages_in_expression(base, counts);
        }
        Expression::FunctionCall(_, func, args) => {
            count_usages_in_expression(func, counts);
            for arg in args {
                count_usages_in_expression(arg, counts);
            }
        }
        Expression::Add(_, a, b)
        | Expression::Subtract(_, a, b)
        | Expression::Multiply(_, a, b)
        | Expression::Divide(_, a, b)
        | Expression::Modulo(_, a, b)
        | Expression::Equal(_, a, b)
        | Expression::NotEqual(_, a, b)
        | Expression::Less(_, a, b)
        | Expression::LessEqual(_, a, b)
        | Expression::More(_, a, b)
        | Expression::MoreEqual(_, a, b)
        | Expression::And(_, a, b)
        | Expression::Or(_, a, b)
        | Expression::BitwiseAnd(_, a, b)
        | Expression::BitwiseOr(_, a, b)
        | Expression::BitwiseXor(_, a, b)
        | Expression::ShiftLeft(_, a, b)
        | Expression::ShiftRight(_, a, b) => {
            count_usages_in_expression(a, counts);
            count_usages_in_expression(b, counts);
        }
        Expression::Not(_, e)
        | Expression::BitwiseNot(_, e)
        | Expression::Negate(_, e)
        | Expression::UnaryPlus(_, e)
        | Expression::PreIncrement(_, e)
        | Expression::PostIncrement(_, e)
        | Expression::PreDecrement(_, e)
        | Expression::PostDecrement(_, e)
        | Expression::Parenthesis(_, e) => {
            count_usages_in_expression(e, counts);
        }
        Expression::ConditionalOperator(_, cond, then_expr, else_expr) => {
            count_usages_in_expression(cond, counts);
            count_usages_in_expression(then_expr, counts);
            count_usages_in_expression(else_expr, counts);
        }
        Expression::List(_, items) => {
            for (_, item) in items {
                if let Some(param) = item {
                    count_usages_in_expression(&param.ty, counts);
                }
            }
        }
        _ => {}
    }
}

/// Check if a YUL block uses any of the given return variable names
fn yul_block_uses_return_vars(yul: &CollectedYulBlock, return_vars: &HashSet<String>) -> bool {
    for stmt in &yul.statements {
        if yul_statement_uses_return_vars(&stmt.statement, return_vars) {
            return true;
        }
    }
    false
}

/// Check if a YUL statement uses any return variable names
fn yul_statement_uses_return_vars(stmt: &YulStatement, return_vars: &HashSet<String>) -> bool {
    match stmt {
        YulStatement::Assign(_, vars, _) => {
            for var in vars {
                // vars are YulExpression - check if it's a variable
                if let YulExpression::Variable(id) = var {
                    if return_vars.contains(&id.name) {
                        return true;
                    }
                }
            }
            false
        }
        YulStatement::Block(block) => {
            for s in &block.statements {
                if yul_statement_uses_return_vars(s, return_vars) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Generate IR for a variable declaration from a return parameter
fn build_return_var_declaration(info: &ReturnVarInfo) -> Vec<IRElement> {
    let mut ir = vec![];

    // Type
    ir.push(format_expression(&info.param.ty));

    // Storage location
    if let Some(storage) = &info.param.storage {
        ir.push(IRElement::text(" "));
        ir.push(format_storage_location(storage));
    }

    // Output name
    ir.push(IRElement::text(" "));
    ir.push(IRElement::text(&info.output_name));
    ir.push(IRElement::text(";"));

    ir
}

/// Build structured IR for function NatSpec comments
/// Takes natspec lines and function parameters to add @param TODO for undocumented params
/// Also adds @return _ TODO if function has returns but no existing @return docs
fn build_function_natspec_ir(
    natspec_lines: &[String],
    func_params: &[(Loc, Option<Parameter>)],
    func_returns: &[(Loc, Option<Parameter>)],
) -> Vec<IRElement> {
    let mut ir = vec![];

    // Start comment block
    ir.push(IRElement::text("/**"));

    // HardLineBreak at start ensures first line gets indented
    let mut content_lines = vec![IRElement::HardLineBreak];

    // Process the natspec lines
    // Track context for continuation lines (lines that don't start with @)
    #[derive(PartialEq)]
    enum NatSpecContext {
        Description,
        Notice,
        Param,
        Return,
        Other,
    }

    let mut description_lines = Vec::new();
    let mut notice_lines = Vec::new();
    let mut param_docs = Vec::new();
    let mut documented_params: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut return_docs = Vec::new();
    let mut other_tags = Vec::new();
    let mut context = NatSpecContext::Description;

    for line in natspec_lines {
        let trimmed = line.trim();

        if trimmed.starts_with("@notice ") {
            let notice_text = trimmed.strip_prefix("@notice ").unwrap_or("").trim();
            notice_lines.push(notice_text.to_string());
            context = NatSpecContext::Notice;
        } else if trimmed.starts_with("@param ") {
            param_docs.push(trimmed.to_string());
            // Extract parameter name to track documented params
            if let Some(rest) = trimmed.strip_prefix("@param ") {
                if let Some(name) = rest.split_whitespace().next() {
                    documented_params.insert(name.to_string());
                }
            }
            context = NatSpecContext::Param;
        } else if trimmed.starts_with("@return ") {
            // Transform @return docs to have _ placeholder after @return
            // Strip the named return variable if present (lowercase identifiers)
            // Input: @return results Array of return data from each call
            // Output: @return _ Array of return data from each call
            // But keep: @return The commitment... -> @return _ The commitment...
            let return_content = trimmed.strip_prefix("@return ").unwrap_or("").trim();
            if return_content.starts_with("_ ") || return_content.starts_with("_\t") {
                // Already has underscore placeholder
                return_docs.push(trimmed.to_string());
            } else {
                // Check if first word is a return variable name (lowercase identifier)
                let mut parts = return_content.splitn(2, char::is_whitespace);
                let first_word = parts.next().unwrap_or("");
                let rest = parts.next().unwrap_or("").trim();

                // Only strip if first word is lowercase (variable name) and there's more content
                let first_char = first_word.chars().next();
                let is_variable_name = first_char.map(|c| c.is_lowercase()).unwrap_or(false);

                if is_variable_name && !rest.is_empty() {
                    // First word was a variable name, use only the description
                    return_docs.push(format!("@return _ {}", rest));
                } else {
                    // First word is part of description (uppercase/article), keep it
                    return_docs.push(format!("@return _ {}", return_content));
                }
            }
            context = NatSpecContext::Return;
        } else if trimmed.starts_with("@custom:semver") {
            // Drop @custom:semver for functions (only relevant for version constants)
            context = NatSpecContext::Other;
        } else if trimmed.starts_with("@dev ") {
            // Combine @dev content into the description (strip @dev prefix)
            let dev_text = trimmed.strip_prefix("@dev ").unwrap_or("").trim();
            description_lines.push(dev_text.to_string());
            context = NatSpecContext::Description;
        } else if trimmed.starts_with("@") {
            other_tags.push(trimmed.to_string());
            context = NatSpecContext::Other;
        } else if !trimmed.is_empty() {
            // Continuation line - append to the appropriate context
            match context {
                NatSpecContext::Description | NatSpecContext::Notice => {
                    description_lines.push(trimmed.to_string());
                }
                NatSpecContext::Param => {
                    // Append to the last @param doc
                    if let Some(last) = param_docs.last_mut() {
                        last.push(' ');
                        last.push_str(trimmed);
                    } else {
                        description_lines.push(trimmed.to_string());
                    }
                }
                NatSpecContext::Return => {
                    // Append to the last @return doc
                    if let Some(last) = return_docs.last_mut() {
                        last.push(' ');
                        last.push_str(trimmed);
                    } else {
                        description_lines.push(trimmed.to_string());
                    }
                }
                NatSpecContext::Other => {
                    // Append to the last other tag
                    if let Some(last) = other_tags.last_mut() {
                        last.push(' ');
                        last.push_str(trimmed);
                    } else {
                        description_lines.push(trimmed.to_string());
                    }
                }
            }
        }
    }

    // Add @param TODO for undocumented parameters (using normalized names)
    // Skip unnamed parameters - they can't be documented
    for (_, param) in func_params {
        if let Some(p) = param {
            // Only consider parameters that have a name
            if let Some(name) = &p.name {
                let original_name = name.name.as_str();
                let normalized_name = normalize_param_name(original_name);
                // Check if documented under either original or normalized name
                if !documented_params.contains(original_name)
                    && !documented_params.contains(&normalized_name)
                {
                    param_docs.push(format!("@param {} TODO", normalized_name));
                }
            }
        }
    }

    // Add notice/description (combine @notice and @dev)
    if !notice_lines.is_empty() || !description_lines.is_empty() {
        let mut all_desc_lines = notice_lines;
        all_desc_lines.extend(description_lines);

        // Combine description into a single line and use word breaks
        let combined_desc = all_desc_lines.join(" ");
        content_lines.push(IRElement::group(text_with_word_breaks(&combined_desc)));
        content_lines.push(IRElement::HardLineBreak);

        // Add blank line before params/returns if there will be any
        // Note: we check func_returns to catch auto-generated @return _ TODO case
        let will_have_return_docs = !return_docs.is_empty() || !func_returns.is_empty();
        if !param_docs.is_empty() || will_have_return_docs || !other_tags.is_empty() {
            content_lines.push(IRElement::HardLineBreak); // blank line before params
        }
    }

    // Add parameter documentation
    let had_any_param_docs = !param_docs.is_empty();
    for param_doc in param_docs {
        // For @param, we want to keep "@param name" together and only break the description
        // Continuation lines should be indented 2 extra spaces
        if let Some((param_part, desc)) = param_doc.split_once("  ") {
            // Handle multiple spaces in the original (e.g., "@param _data   Data to emit...")
            // Normalize the parameter name
            let param_part = param_part.trim();
            let normalized_param_part = if let Some(name) = param_part.strip_prefix("@param ") {
                format!("@param {}", normalize_param_name(name))
            } else {
                param_part.to_string()
            };
            let mut parts = vec![IRElement::text(&normalized_param_part)];
            // Wrap continuation in Indent so wrapping gets extra indentation
            let mut continuation = vec![IRElement::SoftLineBreak];
            continuation.extend(text_with_word_breaks(desc.trim()));
            parts.push(IRElement::indent(continuation));
            content_lines.push(IRElement::group(parts));
        } else if param_doc.contains(" ") {
            // Single space between param name and description
            let parts: Vec<&str> = param_doc.splitn(3, ' ').collect();
            if parts.len() >= 3 {
                // @param name description... - normalize the parameter name
                let normalized_name = normalize_param_name(parts[1]);
                let param_tag_and_name = format!("{} {}", parts[0], normalized_name);
                let mut group_parts = vec![IRElement::text(&param_tag_and_name)];
                // Wrap continuation in Indent so wrapping gets extra indentation
                let mut continuation = vec![IRElement::SoftLineBreak];
                continuation.extend(text_with_word_breaks(parts[2]));
                group_parts.push(IRElement::indent(continuation));
                content_lines.push(IRElement::group(group_parts));
            } else {
                content_lines.push(IRElement::text(&param_doc));
            }
        } else {
            content_lines.push(IRElement::text(&param_doc));
        }
        content_lines.push(IRElement::HardLineBreak);
    }

    // Add return documentation
    // If function has returns but no existing @return docs, add @return _ TODO
    let has_returns = !func_returns.is_empty();
    let has_return_docs = !return_docs.is_empty();

    if has_returns {
        // Only add blank line before @return if there were param docs before it
        // (otherwise we already have a blank line from the description section)
        if had_any_param_docs {
            content_lines.push(IRElement::HardLineBreak); // blank line before @return
        }

        if has_return_docs {
            for return_doc in return_docs {
                // Split @return prefix from description for proper wrapping
                // Format: "@return _ Description text..."
                if let Some(rest) = return_doc.strip_prefix("@return ") {
                    if let Some((name, desc)) = rest.split_once(' ') {
                        // @return name desc
                        let prefix = format!("@return {} ", name);
                        let desc_ir = text_with_word_breaks(desc);
                        let mut return_ir = vec![IRElement::text(&prefix)];
                        return_ir.push(IRElement::indent(vec![IRElement::group(desc_ir)]));
                        content_lines.push(IRElement::group(return_ir));
                    } else {
                        // Just @return name
                        content_lines.push(IRElement::text(&return_doc));
                    }
                } else {
                    content_lines.push(IRElement::text(&return_doc));
                }
                content_lines.push(IRElement::HardLineBreak);
            }
        } else {
            // Auto-add @return _ TODO for functions with returns but no docs
            content_lines.push(IRElement::text("@return _ TODO"));
            content_lines.push(IRElement::HardLineBreak);
        }
    }

    // Add other tags
    for tag in other_tags {
        content_lines.push(IRElement::text(&tag));
        content_lines.push(IRElement::HardLineBreak);
    }

    // Remove trailing line break
    if !content_lines.is_empty() {
        content_lines.pop();
    }

    // Wrap content in indent
    ir.push(IRElement::indent(content_lines));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Build IR for a function definition
pub fn build_function_ir(func: &CollectedFunction) -> Vec<IRElement> {
    let mut ir = vec![];

    // Process leading comments - check if they form a NatSpec comment group
    let leading_comments = &func.definition.leading_comments;

    if !leading_comments.is_empty() {
        // Check if these are NatSpec comments (either block or consecutive line comments with @)
        let mut natspec_lines = Vec::new();
        let mut is_natspec = false;
        let mut has_regular_comments = false;
        let mut current_line = String::new();
        let mut original_doc_block: Option<String> = None;

        for comment in leading_comments {
            match comment {
                Comment::DocLine(_, content) => {
                    let trimmed = content.trim_start_matches("///").trim();
                    if trimmed.contains("@") {
                        is_natspec = true;
                        // If we have accumulated text, push it first
                        if !current_line.is_empty() {
                            natspec_lines.push(current_line.clone());
                            current_line.clear();
                        }
                        current_line.push_str(trimmed);
                    } else if !trimmed.is_empty() {
                        // DocLine without @ tag - still valid NatSpec (description only)
                        is_natspec = true;
                        // Continuation of previous line
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(trimmed);
                    } else {
                        // Empty line - push current and start fresh
                        if !current_line.is_empty() {
                            natspec_lines.push(current_line.clone());
                            current_line.clear();
                        }
                    }
                }
                Comment::DocBlock(_, content) => {
                    // Store original doc block content for @custom:preserve
                    original_doc_block = Some(content.clone());

                    // Remove comment delimiters
                    let inner = content.trim();
                    let inner = if inner.starts_with("/**") && inner.ends_with("*/") {
                        inner
                            .strip_prefix("/**")
                            .unwrap()
                            .strip_suffix("*/")
                            .unwrap()
                            .trim()
                    } else {
                        inner
                    };

                    if content.contains("@") {
                        // This is a NatSpec block with tags - extract lines and process through natspec_lines
                        // so we get proper function-aware handling with continuation support
                        is_natspec = true;

                        // Extract lines, handling continuation
                        for line in inner.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("@") {
                                // New tag - push previous line if any
                                if !current_line.is_empty() {
                                    natspec_lines.push(current_line.clone());
                                    current_line.clear();
                                }
                                current_line.push_str(trimmed);
                            } else if !trimmed.is_empty() {
                                // Continuation line
                                if !current_line.is_empty() {
                                    current_line.push(' ');
                                }
                                current_line.push_str(trimmed);
                            } else if !current_line.is_empty() {
                                // Empty line - push current
                                natspec_lines.push(current_line.clone());
                                current_line.clear();
                            }
                        }
                        // Don't continue here - let the natspec_lines processing handle it
                    } else {
                        // DocBlock without @ tags - treat as description-only
                        // Collect all non-empty lines as description
                        for line in inner.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                if !current_line.is_empty() {
                                    current_line.push(' ');
                                }
                                current_line.push_str(trimmed);
                            }
                        }
                        // Mark as natspec so it gets output
                        if !current_line.is_empty() {
                            is_natspec = true;
                        }
                    }
                }
                Comment::Line(_, content) => {
                    // Regular line comment - collect content for potential NatSpec conversion
                    has_regular_comments = true;
                    let trimmed = content.trim_start_matches("//").trim();
                    if !trimmed.is_empty() {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(trimmed);
                    }
                }
                _ => {
                    // Other comment types
                    ir.push(build_comment(comment));
                    ir.push(IRElement::HardLineBreak);
                }
            }
        }

        // Push any remaining content (clone to preserve for regular comment check)
        if !current_line.is_empty() && is_natspec {
            natspec_lines.push(current_line.clone());
        }

        // If we collected NatSpec lines, format them appropriately
        if is_natspec && !natspec_lines.is_empty() {
            // Check for @custom:preserve - if present, output the original comment unchanged
            let has_preserve = natspec_lines.iter().any(|l| {
                let t = l.trim();
                t == "@custom:preserve" || t.starts_with("@custom:preserve ")
            });

            if has_preserve {
                // Preserve mode - output the original comment unchanged
                // Use IRElement::comment which goes through print_comment's preserve handling
                if let Some(content) = &original_doc_block {
                    // Strip /** and */ delimiters but preserve internal formatting
                    let inner = content.trim();
                    let inner = if inner.starts_with("/**") && inner.ends_with("*/") {
                        inner
                            .strip_prefix("/**")
                            .unwrap()
                            .strip_suffix("*/")
                            .unwrap()
                            .trim()
                    } else {
                        inner
                    };
                    ir.push(IRElement::comment(inner, true));
                    ir.push(IRElement::HardLineBreak);
                } else {
                    // DocLine comments - output them as-is
                    for comment in leading_comments {
                        ir.push(build_comment(comment));
                        ir.push(IRElement::HardLineBreak);
                    }
                }
            } else {
                // Check if function needs block format (has params/returns) or can use single line
                let has_params = !func.definition.element.params.is_empty();
                let has_returns = !func.definition.element.returns.is_empty();
                let has_param_or_return_docs = natspec_lines
                    .iter()
                    .any(|l| l.trim().starts_with("@param ") || l.trim().starts_with("@return "));
                let has_other_tags = natspec_lines.iter().any(|l| {
                    let t = l.trim();
                    t.starts_with("@")
                        && !t.starts_with("@notice")
                        && !t.starts_with("@custom:semver")
                });

                if has_params || has_returns || has_param_or_return_docs || has_other_tags {
                    // Needs block format
                    ir.extend(build_function_natspec_ir(
                        &natspec_lines,
                        &func.definition.element.params,
                        &func.definition.element.returns,
                    ));
                    ir.push(IRElement::HardLineBreak);
                } else {
                    // Simple @notice only - use single line format with notice stripped
                    let description: String = natspec_lines
                        .iter()
                        .map(|line| {
                            let trimmed = line.trim();
                            trimmed.strip_prefix("@notice ").unwrap_or(trimmed)
                        })
                        .collect::<Vec<_>>()
                        .join(" ");
                    ir.push(IRElement::comment(description, true));
                    ir.push(IRElement::HardLineBreak);
                }
            }
        } else if !has_regular_comments && !natspec_lines.is_empty() {
            // Doc comments without NatSpec tags - just output as-is
            for comment in leading_comments {
                ir.push(build_comment(comment));
                ir.push(IRElement::HardLineBreak);
            }
        } else if has_regular_comments
            && !current_line.is_empty()
            && matches!(
                func.definition.element.ty,
                FunctionTy::Function | FunctionTy::Constructor | FunctionTy::Modifier
            )
        {
            // Regular comments on a function - convert to NatSpec with the comment as description
            ir.extend(generate_function_natspec_with_description(
                &func,
                Some(&current_line),
            ));
            ir.push(IRElement::HardLineBreak);
        }
    } else if matches!(
        func.definition.element.ty,
        FunctionTy::Function | FunctionTy::Constructor | FunctionTy::Modifier
    ) {
        // No documentation for public functions/modifiers - generate NatSpec
        ir.extend(generate_function_natspec(&func));
        ir.push(IRElement::HardLineBreak);
    }

    // Build the function definition
    let function_ir = build_function_definition(func);
    ir.extend(function_ir);

    // Move trailing comments to the next line as leading comments
    if !func.definition.trailing_comments.is_empty() {
        ir.push(IRElement::HardLineBreak);
        for comment in &func.definition.trailing_comments {
            ir.push(build_comment(comment));
            ir.push(IRElement::HardLineBreak);
        }
    }

    ir
}

fn build_function_definition(func: &CollectedFunction) -> Vec<IRElement> {
    let mut ir = vec![];

    // Build rename map for parameters
    let param_renames = build_param_rename_map(&func.definition.element.params);

    // Function type (function, modifier, fallback, receive, constructor)
    match &func.definition.element.ty {
        FunctionTy::Constructor => ir.push(IRElement::text("constructor")),
        FunctionTy::Function => {
            ir.push(IRElement::text("function"));
            ir.push(IRElement::text(" "));
            if let Some(name) = &func.definition.element.name {
                ir.push(IRElement::text(&name.name));
            }
        }
        FunctionTy::Fallback => ir.push(IRElement::text("fallback")),
        FunctionTy::Receive => ir.push(IRElement::text("receive")),
        FunctionTy::Modifier => {
            ir.push(IRElement::text("modifier"));
            ir.push(IRElement::text(" "));
            if let Some(name) = &func.definition.element.name {
                ir.push(IRElement::text(&name.name));
            }
        }
    }

    // Parameters - space before ( for readability
    ir.push(IRElement::text(" ("));
    if !func.definition.element.params.is_empty() {
        // Always put parameters on separate lines
        let mut params_content = vec![];
        for (i, param) in func.definition.element.params.iter().enumerate() {
            if i > 0 {
                params_content.push(IRElement::text(","));
            }
            params_content.push(IRElement::HardLineBreak);
            if let Some(p) = &param.1 {
                params_content.extend(format_parameter(p));
            }
        }
        ir.push(IRElement::indent(params_content));
        ir.push(IRElement::HardLineBreak);
    }
    ir.push(IRElement::text(")"));

    // Function attributes (visibility, mutability, modifiers, etc.) and returns
    // are wrapped together in a Group so that when the full signature exceeds
    // the line limit, the returns clause will break appropriately.
    let mut signature_tail = vec![];

    // Determine if we have a body and opening brace
    let has_body = func.definition.element.body.is_some();
    let is_empty_body = func.body_statements.is_empty()
        || (func.body_statements.len() == 1
            && matches!(&func.body_statements[0].statement, Statement::Block { statements, .. } if statements.is_empty()));
    let has_returns = !func.definition.element.returns.is_empty();

    // Build attributes with continuation break support. Wrap in a Group so that:
    // - When attrs fit on line with function name, render flat (space before attrs)
    // - When attrs DON'T fit, break with continuation indent before attrs
    // Include the opening brace in the attrs group when there's no returns clause,
    // so that `public {` stays together and breaks as a unit.
    if !func.definition.element.attributes.is_empty() {
        let mut attrs_content = vec![];
        for (i, attr) in func.definition.element.attributes.iter().enumerate() {
            if i > 0 {
                attrs_content.push(IRElement::text(" "));
            }
            attrs_content.push(format_function_attribute(attr, &param_renames));
        }
        // Include opening brace with attrs when no returns clause
        if !has_returns && has_body {
            if is_empty_body {
                attrs_content.push(IRElement::text(" { }"));
            } else {
                attrs_content.push(IRElement::text(" {"));
            }
        }
        // SoftLineBreakWithContinuation becomes space in flat mode, or newline+indent when breaking
        signature_tail.push(IRElement::group(vec![
            IRElement::SoftLineBreakWithContinuation,
            IRElement::group(attrs_content),
        ]));
    }

    // Return parameters
    if !func.definition.element.returns.is_empty() {
        let mut returns_ir = vec![];
        for (i, ret) in func.definition.element.returns.iter().enumerate() {
            if i > 0 {
                returns_ir.push(IRElement::text(", "));
            }

            if let Some(r) = &ret.1 {
                // Strip return parameter names entirely
                returns_ir.extend(format_parameter_with_options(r, false, true));
            }
        }

        // Add returns with SoftestLineBreak that will trigger when the outer group breaks
        // SoftestLineBreak becomes nothing when not breaking (keeping "returns (type)")
        // but becomes a newline when the group needs to break
        signature_tail.push(IRElement::text(" returns ("));
        signature_tail.push(IRElement::indent(vec![
            IRElement::SoftestLineBreak,
            IRElement::group(returns_ir),
        ]));
        signature_tail.push(IRElement::SoftestLineBreak);
        signature_tail.push(IRElement::text(")"));
    }

    // Function body - include the opening brace in the signature group
    // so the full line length is measured when deciding to break.
    // Only add here if we haven't already added it with the attrs (which happens
    // when there are attrs but no returns clause).
    let brace_already_added =
        !func.definition.element.attributes.is_empty() && !has_returns && has_body;

    if has_body && !brace_already_added {
        if is_empty_body {
            // Empty body: use "{ }" on single line
            signature_tail.push(IRElement::text(" { }"));
        } else {
            signature_tail.push(IRElement::text(" {"));
        }
    }

    // Wrap the attributes + returns + brace in a Group so line length is evaluated together
    if !signature_tail.is_empty() {
        ir.push(IRElement::group(signature_tail));
    }

    // Add semicolon for bodyless functions, or continue with body content
    if !has_body {
        ir.push(IRElement::text(";"));
        return ir;
    }

    if is_empty_body {
        return ir;
    }

    // Function body content (the opening brace is already in the signature group)

    // Build map of return variable names to their types for return var handling
    let mut return_var_types: HashMap<String, Parameter> = HashMap::new();
    let mut return_vars: HashSet<String> = HashSet::new();
    for (_, ret) in &func.definition.element.returns {
        if let Some(r) = ret {
            if let Some(name) = &r.name {
                return_vars.insert(name.name.clone());
                return_var_types.insert(name.name.clone(), r.clone());
            }
        }
    }

    // Track which return vars have been declared (first assignment becomes declaration)
    let mut declared_return_vars: HashSet<String> = HashSet::new();

    // Count how many times each return var is used in the function body
    // Variables used only once (just the assignment) can use direct `return expr;`
    // BUT only if there's exactly one return variable (otherwise we need tuple return)
    let return_var_usage_counts = count_return_var_usages(&func.body_statements, &return_vars);
    let single_assignment_vars: HashSet<String> = if return_vars.len() == 1 {
        return_var_usage_counts
            .iter()
            .filter(|(_, count)| **count == 1)
            .map(|(name, _)| name.clone())
            .collect()
    } else {
        // Multiple return vars - can't use direct return
        HashSet::new()
    };

    // Build return var info map (with types) for assembly transformation
    let return_var_info = build_return_var_info(&func.definition.element.returns);

    // Build rename map for return vars used in assembly
    let mut assembly_renames: HashMap<String, String> = HashMap::new();
    for (name, info) in &return_var_info {
        assembly_renames.insert(name.clone(), info.output_name.clone());
    }

    // Combine param renames with assembly renames for statement processing
    let mut all_renames = param_renames.clone();
    all_renames.extend(assembly_renames.clone());

    // Track which return vars are actually used in assembly
    let mut used_in_assembly: HashSet<String> = HashSet::new();

    // Use the collected body statements for proper formatting
    if !func.body_statements.is_empty() {
        ir.push(IRElement::HardLineBreak);

        let mut body_ir = vec![];

        // Get statements to process (may be nested in a block)
        let stmts_to_process: &[CommentedStatement] = if func.body_statements.len() == 1 {
            if let Statement::Block { .. } = &func.body_statements[0].statement {
                if let Some(nested) = &func.body_statements[0].nested_statements {
                    nested
                } else {
                    &func.body_statements
                }
            } else {
                &func.body_statements
            }
        } else {
            &func.body_statements
        };

        // Add blank line after opening brace if the first statement has leading comments
        let first_has_comments = stmts_to_process
            .first()
            .map(|s| !s.leading_comments.is_empty())
            .unwrap_or(false);
        if first_has_comments {
            ir.push(IRElement::HardLineBreak);
        }

        for (i, stmt) in stmts_to_process.iter().enumerate() {
            if i > 0 {
                body_ir.push(IRElement::HardLineBreak);
                if needs_blank_line_before_statement(stmts_to_process, i) {
                    body_ir.push(IRElement::HardLineBreak);
                }
            }

            // Check if this is an assembly statement that uses return vars
            if let Statement::Assembly {
                loc,
                dialect,
                flags,
                block: _,
            } = &stmt.statement
            {
                if let Some(yul) = &stmt.yul_block {
                    if yul_block_uses_return_vars(yul, &return_vars) {
                        // For assembly that uses return vars, we need to:
                        // 1. Emit leading comments first
                        // 2. Emit the variable declaration
                        // 3. Emit the assembly without its leading comments

                        // Emit leading comments
                        body_ir.extend(build_leading_comments(&stmt.leading_comments));

                        // Emit variable declarations for used return vars
                        for var_name in &return_vars {
                            if let Some(info) = return_var_info.get(var_name) {
                                if !used_in_assembly.contains(var_name) {
                                    body_ir.extend(build_return_var_declaration(info));
                                    body_ir.push(IRElement::HardLineBreak);
                                    used_in_assembly.insert(var_name.clone());
                                }
                            }
                        }

                        // Emit the assembly statement without leading comments (but with trailing)
                        body_ir.extend(with_comments(&[], &stmt.trailing_comments, || {
                            format_assembly_with_collected_and_renames(
                                loc,
                                dialect.as_ref(),
                                flags.as_ref(),
                                yul,
                                &mut all_renames,
                            )
                        }));
                        continue;
                    }
                }
            }

            // Process statement with renames applied
            let mut return_ctx = ReturnVarContext {
                return_var_types: &return_var_types,
                declared_return_vars: &mut declared_return_vars,
                single_assignment_vars: &single_assignment_vars,
            };
            body_ir.extend(build_statement_ir_full(stmt, &mut all_renames, &mut return_ctx));
        }

        // Add explicit return for variables used in assembly
        if !used_in_assembly.is_empty() {
            body_ir.push(IRElement::HardLineBreak);

            // Build return expression for all used vars
            let return_exprs: Vec<String> = used_in_assembly
                .iter()
                .filter_map(|name| {
                    return_var_info
                        .get(name)
                        .map(|info| info.output_name.clone())
                })
                .collect();

            body_ir.push(IRElement::text("return "));
            body_ir.push(IRElement::text(&return_exprs.join(", ")));
            body_ir.push(IRElement::text(";"));
        } else if !declared_return_vars.is_empty() {
            // Add explicit return for declared return vars
            body_ir.push(IRElement::HardLineBreak);

            // Build return expression - maintain declaration order and use normalized names
            let return_exprs: Vec<String> = func
                .definition
                .element
                .returns
                .iter()
                .filter_map(|(_, ret)| {
                    ret.as_ref().and_then(|r| {
                        r.name.as_ref().and_then(|name| {
                            if declared_return_vars.contains(&name.name) {
                                // Use normalized name from renames map, or normalize directly
                                Some(
                                    all_renames
                                        .get(&name.name)
                                        .cloned()
                                        .unwrap_or_else(|| normalize_param_name(&name.name)),
                                )
                            } else {
                                None
                            }
                        })
                    })
                })
                .collect();

            if return_exprs.len() == 1 {
                body_ir.push(IRElement::text(&format!("return {};", return_exprs[0])));
            } else {
                body_ir.push(IRElement::text(&format!(
                    "return ({});",
                    return_exprs.join(", ")
                )));
            }
        }

        ir.push(IRElement::indent(body_ir));
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text("}"));

    ir
}

// Removed - using format_parameter from expressions.rs instead

fn format_function_attribute(
    attr: &FunctionAttribute,
    renames: &HashMap<String, String>,
) -> IRElement {
    match attr {
        FunctionAttribute::Visibility(vis) => format_visibility(vis),
        FunctionAttribute::Mutability(mut_) => format_mutability(mut_),
        FunctionAttribute::Virtual(_) => IRElement::text("virtual"),
        FunctionAttribute::Immutable(_) => IRElement::text("immutable"),
        FunctionAttribute::Override(_, paths) => {
            if paths.is_empty() {
                IRElement::text("override")
            } else {
                let mut ir = vec![IRElement::text("override(")];
                for (i, path) in paths.iter().enumerate() {
                    if i > 0 {
                        ir.push(IRElement::text(", "));
                    }
                    ir.push(format_identifier_path(path));
                }
                ir.push(IRElement::text(")"));
                IRElement::group(ir)
            }
        }
        FunctionAttribute::BaseOrModifier(_, base) => format_base_or_modifier(base, renames),
        FunctionAttribute::Error(_) => IRElement::text("/* error */"),
    }
}

fn format_mutability(mut_: &Mutability) -> IRElement {
    IRElement::text(match mut_ {
        Mutability::Pure(_) => "pure",
        Mutability::View(_) => "view",
        Mutability::Constant(_) => "constant",
        Mutability::Payable(_) => "payable",
    })
}

fn format_base_or_modifier(base: &Base, renames: &HashMap<String, String>) -> IRElement {
    let mut ir = vec![format_identifier_path(&base.name)];

    if let Some(args) = &base.args {
        ir.push(IRElement::text("("));
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                ir.push(IRElement::text(", "));
            }
            ir.push(format_expression_with_renames(arg, renames));
        }
        ir.push(IRElement::text(")"));
    }

    IRElement::group(ir)
}

/// Generate NatSpec documentation for an undocumented function
/// If `description` is provided, use it instead of "TODO"
fn generate_function_natspec_with_description(
    func: &CollectedFunction,
    description: Option<&str>,
) -> Vec<IRElement> {
    let mut ir = vec![];

    ir.push(IRElement::text("/**"));

    // HardLineBreak at start ensures first line gets indented
    let mut content = vec![IRElement::HardLineBreak];

    // Add description or TODO notice
    let desc_text = description.unwrap_or("TODO");
    content.push(IRElement::text(desc_text));
    content.push(IRElement::HardLineBreak);

    // Add blank line before parameters if there are any
    let has_params = !func.definition.element.params.is_empty();
    let has_returns = !func.definition.element.returns.is_empty();

    if has_params || has_returns {
        content.push(IRElement::HardLineBreak);

        // Add parameter documentation (using normalized names)
        for (_, param) in &func.definition.element.params {
            if let Some(p) = param {
                let param_name = p
                    .name
                    .as_ref()
                    .map(|n| n.name.as_str())
                    .unwrap_or("unnamed");
                let normalized_name = normalize_param_name(param_name);
                content.push(IRElement::text(&format!("@param {} TODO", normalized_name)));
                content.push(IRElement::HardLineBreak);
            }
        }

        // Add return documentation with blank line before (only if there were params)
        if has_returns {
            if has_params {
                content.push(IRElement::HardLineBreak); // blank line before @return only if params exist
            }
            content.push(IRElement::text("@return _ TODO"));
            content.push(IRElement::HardLineBreak);
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

/// Generate NatSpec documentation for an undocumented function
fn generate_function_natspec(func: &CollectedFunction) -> Vec<IRElement> {
    generate_function_natspec_with_description(func, None)
}
