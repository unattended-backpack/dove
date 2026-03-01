//! Contract definition builders for the IR

use super::ir::IRElement;
use crate::ir_builder::comments::{build_comment, build_natspec_comment_ir};
use crate::ir_builder::expressions::*;
use crate::ir_builder::functions::*;
use crate::ir_builder::text_with_word_breaks;
use crate::ir_builder::toplevel::*;
use crate::ir_builder::types::*;
use crate::collector::model::*;
use solang_parser::pt::*;
use std::collections::{HashMap, HashSet};

/// Extract variable names referenced in an expression (for array size expressions)
fn extract_referenced_vars(expr: &Expression) -> HashSet<String> {
    let mut vars = HashSet::new();
    collect_vars_from_expr(expr, &mut vars);
    vars
}

fn collect_vars_from_expr(expr: &Expression, vars: &mut HashSet<String>) {
    match expr {
        Expression::Variable(ident) => {
            vars.insert(ident.name.clone());
        }
        Expression::ArraySubscript(_, base, size) => {
            collect_vars_from_expr(base, vars);
            if let Some(s) = size {
                collect_vars_from_expr(s, vars);
            }
        }
        Expression::MemberAccess(_, base, _) => {
            collect_vars_from_expr(base, vars);
        }
        Expression::Add(_, a, b)
        | Expression::Subtract(_, a, b)
        | Expression::Multiply(_, a, b)
        | Expression::Divide(_, a, b)
        | Expression::Modulo(_, a, b) => {
            collect_vars_from_expr(a, vars);
            collect_vars_from_expr(b, vars);
        }
        Expression::Parenthesis(_, inner) => {
            collect_vars_from_expr(inner, vars);
        }
        _ => {}
    }
}

/// Find all constants that a struct depends on (for array size expressions)
/// Returns them in field order (order they first appear in struct fields)
fn find_struct_constant_deps_ordered(struct_def: &CollectedStruct) -> Vec<String> {
    let mut deps = Vec::new();
    let mut seen = HashSet::new();
    for field in &struct_def.fields {
        let mut field_deps = HashSet::new();
        collect_type_constant_deps(&field.element.ty, &mut field_deps);
        // Add deps in a consistent order (but maintain field order for first occurrences)
        for dep in field_deps {
            if !seen.contains(&dep) {
                seen.insert(dep.clone());
                deps.push(dep);
            }
        }
    }
    deps
}

/// Collect constant references from a type expression
fn collect_type_constant_deps(ty: &Expression, deps: &mut HashSet<String>) {
    match ty {
        Expression::ArraySubscript(_, base, size) => {
            // The base type might itself be an array with constants
            collect_type_constant_deps(base, deps);
            // The size expression references constants
            if let Some(size_expr) = size {
                deps.extend(extract_referenced_vars(size_expr));
            }
        }
        Expression::MemberAccess(_, base, _) => {
            collect_type_constant_deps(base, deps);
        }
        _ => {}
    }
}

/// Find dependencies of a constant (other constants it references in its initializer)
fn find_constant_deps(var: &CommentedElement<Box<VariableDefinition>>) -> HashSet<String> {
    let mut deps = HashSet::new();
    if let Some(init) = &var.element.initializer {
        collect_vars_from_expr(init, &mut deps);
    }
    deps
}

/// Resolve transitive dependencies: given an ordered list of required constants,
/// return them in dependency order (dependencies before dependents), preserving
/// the relative order of the input list where possible.
fn resolve_constant_deps_order(
    required: &[String],
    constants: &HashMap<String, &CommentedElement<Box<VariableDefinition>>>,
) -> Vec<String> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut in_progress = HashSet::new();

    fn visit(
        name: &str,
        constants: &HashMap<String, &CommentedElement<Box<VariableDefinition>>>,
        result: &mut Vec<String>,
        visited: &mut HashSet<String>,
        in_progress: &mut HashSet<String>,
    ) {
        if visited.contains(name) || in_progress.contains(name) {
            return;
        }
        in_progress.insert(name.to_string());

        // Find this constant's dependencies
        if let Some(var) = constants.get(name) {
            let deps = find_constant_deps(var);
            // Sort deps for deterministic ordering
            let mut sorted_deps: Vec<_> = deps.into_iter().collect();
            sorted_deps.sort();
            for dep in sorted_deps {
                if constants.contains_key(&dep) {
                    visit(&dep, constants, result, visited, in_progress);
                }
            }
        }

        in_progress.remove(name);
        visited.insert(name.to_string());
        result.push(name.to_string());
    }

    for name in required {
        visit(name, constants, &mut result, &mut visited, &mut in_progress);
    }

    result
}

/// Build structured IR for contract NatSpec comments
fn build_contract_natspec_ir(natspec_lines: &[String], contract_name: &str) -> Vec<IRElement> {
    let mut ir = vec![];

    // Start comment block
    ir.push(IRElement::text("/**"));

    // Content lines - HardLineBreak at start ensures first line gets indented
    let mut content_lines = vec![IRElement::HardLineBreak];

    // Add benediction (always the same)
    content_lines.push(IRElement::text(
        "@custom:benediction DEVS BENEDICAT ET PROTEGAT CONTRACTVM MEVM",
    ));
    content_lines.push(IRElement::HardLineBreak);

    // Process the natspec lines
    let mut has_title = false;
    let mut has_author = false;
    let mut has_custom_terry = false;
    let mut description_lines = Vec::new();
    let mut notice_lines = Vec::new();
    let mut custom_tags_before_desc = Vec::new(); // Custom tags that go before description
    let mut date_tag: Option<String> = None; // @custom:date goes at the end

    for line in natspec_lines {
        let trimmed = line.trim();

        if trimmed.starts_with("@title ") {
            has_title = true;
            content_lines.push(IRElement::text(trimmed));
            content_lines.push(IRElement::HardLineBreak);
        } else if trimmed.starts_with("@author ") {
            has_author = true;
            content_lines.push(IRElement::text(trimmed));
            content_lines.push(IRElement::HardLineBreak);
        } else if trimmed.starts_with("@custom:terry ") {
            has_custom_terry = true;
            content_lines.push(IRElement::text(trimmed));
            content_lines.push(IRElement::HardLineBreak);
        } else if trimmed.starts_with("@custom:date ") {
            date_tag = Some(trimmed.to_string());
        } else if trimmed.starts_with("@notice ") {
            let notice_text = trimmed.strip_prefix("@notice ").unwrap_or("").trim();
            notice_lines.push(notice_text.to_string());
        } else if trimmed.starts_with("@dev ") {
            // @dev content gets combined into the description like @notice
            let dev_text = trimmed.strip_prefix("@dev ").unwrap_or("").trim();
            notice_lines.push(dev_text.to_string());
        } else if trimmed.starts_with("@custom:benediction ") {
            // Skip benediction - we always use the standard one
        } else if trimmed.starts_with("@custom:") || trimmed.starts_with("@") {
            // Other custom tags go before description
            custom_tags_before_desc.push(trimmed.to_string());
        } else if !trimmed.is_empty() {
            // This is a continuation of the previous line - check if we have a notice being built
            if !notice_lines.is_empty() {
                // Append to the last notice line
                let last_notice = notice_lines.last_mut().unwrap();
                last_notice.push(' ');
                last_notice.push_str(trimmed);
            } else {
                description_lines.push(trimmed.to_string());
            }
        }
    }

    // Add missing tags with defaults
    if !has_title {
        content_lines.push(IRElement::text(&format!("@title {}", contract_name)));
        content_lines.push(IRElement::HardLineBreak);
    }
    if !has_author {
        content_lines.push(IRElement::text("@author TODO"));
        content_lines.push(IRElement::HardLineBreak);
    }
    if !has_custom_terry {
        content_lines.push(IRElement::text(
            "@custom:terry \"Is this too much voodoo for the next ten centuries?\"",
        ));
        content_lines.push(IRElement::HardLineBreak);
    }

    // Add other custom tags (before description)
    for tag in &custom_tags_before_desc {
        content_lines.push(IRElement::text(tag));
        content_lines.push(IRElement::HardLineBreak);
    }

    // Add description/notice if present, or TODO if missing
    content_lines.push(IRElement::HardLineBreak); // blank line before description

    if !notice_lines.is_empty() || !description_lines.is_empty() {
        let all_desc_lines = if !notice_lines.is_empty() {
            notice_lines
        } else {
            description_lines
        };

        // Combine description into a single line and use word breaks
        let combined_desc = all_desc_lines.join(" ");
        content_lines.push(IRElement::group(text_with_word_breaks(&combined_desc)));
    } else {
        content_lines.push(IRElement::text("TODO"));
    }
    content_lines.push(IRElement::HardLineBreak);

    // Always add blank line before @custom:date
    content_lines.push(IRElement::HardLineBreak);

    // Add @custom:date at the end (or TODO if missing)
    if let Some(tag) = date_tag {
        content_lines.push(IRElement::text(&tag));
    } else {
        content_lines.push(IRElement::text("@custom:date TODO."));
    }

    // Wrap content in indent
    ir.push(IRElement::indent(content_lines));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

/// Generate placeholder NatSpec for an undocumented contract
fn generate_contract_natspec(contract_name: &str) -> Vec<IRElement> {
    // Just call build_contract_natspec_ir with empty lines - it will fill in all defaults
    build_contract_natspec_ir(&[], contract_name)
}

/// Build IR for a contract definition
pub fn build_contract_ir(contract: &CollectedContract) -> Vec<IRElement> {
    // Handle contract-level comments specially for NatSpec
    let mut ir = vec![];

    // Process leading comments - check if they form a NatSpec comment group
    let leading_comments = &contract.definition.leading_comments;
    let contract_name = contract
        .definition
        .element
        .name
        .as_ref()
        .map(|n| n.name.as_str())
        .unwrap_or("TODO");

    if !leading_comments.is_empty() {
        // Check if these are NatSpec comments (either block or consecutive line comments with @)
        let mut natspec_lines = Vec::new();
        let mut is_natspec = false;
        let mut original_doc_block: Option<&str> = None;

        for comment in leading_comments {
            match comment {
                Comment::DocLine(_, content) => {
                    let trimmed = content.trim_start_matches("///").trim();
                    if trimmed.contains("@") {
                        is_natspec = true;
                    }
                    natspec_lines.push(trimmed.to_string());
                }
                Comment::DocBlock(_, content) => {
                    if content.contains("@") {
                        // Parse the DocBlock into lines for contract NatSpec processing
                        is_natspec = true;
                        original_doc_block = Some(content);
                        let block_content = content
                            .trim()
                            .strip_prefix("/**")
                            .unwrap_or(content.trim())
                            .strip_suffix("*/")
                            .unwrap_or(content.trim())
                            .trim();
                        for line in block_content.lines() {
                            let trimmed = line.trim().trim_start_matches('*').trim();
                            if !trimmed.is_empty() {
                                natspec_lines.push(trimmed.to_string());
                            }
                        }
                    }
                }
                _ => {
                    // Regular comment
                    ir.push(build_comment(comment));
                    ir.push(IRElement::HardLineBreak);
                }
            }
        }

        // If we collected NatSpec lines, check if it's complete or needs filling in
        if is_natspec && !natspec_lines.is_empty() {
            // Check for @custom:preserve - if present, output the original comment unchanged
            let has_preserve = natspec_lines.iter().any(|l| l.trim() == "@custom:preserve" || l.starts_with("@custom:preserve "));

            if has_preserve {
                // Preserve mode - output the original comment unchanged
                if let Some(content) = original_doc_block {
                    ir.extend(build_natspec_comment_ir(content));
                    ir.push(IRElement::HardLineBreak);
                } else {
                    // DocLine comments - output them as-is
                    for comment in leading_comments {
                        ir.push(build_comment(comment));
                        ir.push(IRElement::HardLineBreak);
                    }
                }
            } else {
                // Check if the NatSpec has all required fields
                let has_benediction = natspec_lines.iter().any(|l| l.starts_with("@custom:benediction "));
                let has_title = natspec_lines.iter().any(|l| l.starts_with("@title "));
                let has_author = natspec_lines.iter().any(|l| l.starts_with("@author "));
                let has_terry = natspec_lines.iter().any(|l| l.starts_with("@custom:terry "));
                let has_date = natspec_lines.iter().any(|l| l.starts_with("@custom:date "));
                // Description is any non-empty line that doesn't start with @
                let has_description = natspec_lines.iter().any(|l| !l.is_empty() && !l.starts_with("@"));

                let is_complete = has_benediction && has_title && has_author && has_terry && has_date && has_description;

                if is_complete {
                    // NatSpec is complete - use build_natspec_comment_ir to preserve structure
                    if let Some(content) = original_doc_block {
                        ir.extend(build_natspec_comment_ir(content));
                        ir.push(IRElement::HardLineBreak);
                    } else {
                        // DocLine comments - just output them as-is formatted
                        ir.extend(build_contract_natspec_ir(&natspec_lines, contract_name));
                        ir.push(IRElement::HardLineBreak);
                    }
                } else {
                    // NatSpec is incomplete - fill in missing fields
                    ir.extend(build_contract_natspec_ir(&natspec_lines, contract_name));
                    ir.push(IRElement::HardLineBreak);
                }
            }
        } else if !natspec_lines.is_empty() {
            // Not NatSpec, just regular doc comments
            for comment in leading_comments {
                ir.push(build_comment(comment));
                ir.push(IRElement::HardLineBreak);
            }
        }
    } else {
        // No documentation - generate placeholder NatSpec
        ir.extend(generate_contract_natspec(contract_name));
        ir.push(IRElement::HardLineBreak);
    }

    // Build contract definition without with_comments wrapper
    let contract_ir = build_contract_definition(contract);

    ir.extend(contract_ir);

    // Move trailing comments to the next line as leading comments
    if !contract.definition.trailing_comments.is_empty() {
        ir.push(IRElement::HardLineBreak);
        for comment in &contract.definition.trailing_comments {
            ir.push(build_comment(comment));
            ir.push(IRElement::HardLineBreak);
        }
    }

    ir
}

fn build_contract_definition(contract: &CollectedContract) -> Vec<IRElement> {
    let mut ir = vec![];

    // Contract type (contract, interface, library, abstract)
    match &contract.definition.element.ty {
        ContractTy::Contract(_) => ir.push(IRElement::text("contract")),
        ContractTy::Abstract(_) => {
            ir.push(IRElement::text("abstract"));
            ir.push(IRElement::text(" "));
            ir.push(IRElement::text("contract"));
        }
        ContractTy::Interface(_) => ir.push(IRElement::text("interface")),
        ContractTy::Library(_) => ir.push(IRElement::text("library")),
    }

    ir.push(IRElement::text(" "));
    if let Some(name) = &contract.definition.element.name {
        ir.push(IRElement::text(&name.name));
    }

    // Base contracts - each on its own line, indented
    if !contract.definition.element.base.is_empty() {
        ir.push(IRElement::text(" is"));

        let mut inheritance_ir = vec![];
        for (i, base) in contract.definition.element.base.iter().enumerate() {
            if i > 0 {
                inheritance_ir.push(IRElement::text(","));
            }
            inheritance_ir.push(IRElement::HardLineBreak);
            inheritance_ir.push(format_base(&base));
        }
        inheritance_ir.push(IRElement::text(" "));
        inheritance_ir.push(IRElement::text("{"));

        ir.push(IRElement::indent(inheritance_ir));
    } else {
        ir.push(IRElement::text(" "));
        ir.push(IRElement::text("{"));
    }

    // Contract body - collect all elements into a single indent block
    let has_contents = has_contract_contents(&contract.contents);
    if has_contents {
        let mut body_ir = vec![IRElement::HardLineBreak]; // Initial newline after {

        let mut first_element = true;

        // Using directives
        for using in &contract.contents.using_directives {
            if !first_element {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_using_directive_ir(using));
            first_element = false;
        }

        // Type definitions
        for type_def in &contract.contents.types {
            if !first_element {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_type_definition_ir(type_def));
            first_element = false;
        }

        // Enums
        for enum_def in &contract.contents.enums {
            if !first_element {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_enum_ir(enum_def));
            first_element = false;
        }

        // Build a map of constant names to their definitions
        // (constants are state variables that are declared as constant)
        let constants_map: HashMap<String, &CommentedElement<Box<VariableDefinition>>> = contract
            .contents
            .variables
            .iter()
            .filter(|v| {
                v.element.attrs.iter().any(|attr| {
                    matches!(attr, VariableAttribute::Constant(_))
                })
            })
            .filter_map(|v| {
                v.element.name.as_ref().map(|n| (n.name.clone(), v))
            })
            .collect();

        // Track which constants have been emitted (to avoid duplicates)
        let mut emitted_constants: HashSet<String> = HashSet::new();

        // Structs - emit dependent constants before each struct
        for (i, struct_def) in contract.contents.structs.iter().enumerate() {
            // Find constants this struct depends on (in field order)
            let struct_deps = find_struct_constant_deps_ordered(struct_def);

            // Resolve transitive dependencies and get them in order
            let deps_in_order = resolve_constant_deps_order(&struct_deps, &constants_map);

            // Emit each dependency constant (if not already emitted)
            for dep_name in deps_in_order {
                if !emitted_constants.contains(&dep_name) {
                    if let Some(var_def) = constants_map.get(&dep_name) {
                        if !first_element {
                            body_ir.push(IRElement::HardLineBreak);
                        }
                        body_ir.push(IRElement::HardLineBreak);
                        body_ir.extend(build_variable_definition_ir(var_def));
                        first_element = false;
                        emitted_constants.insert(dep_name);
                    }
                }
            }

            // Emit the struct
            if !first_element || i > 0 {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_struct_ir(struct_def));
            first_element = false;
        }

        // Errors
        for error_def in &contract.contents.errors {
            if !first_element {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_error_ir(error_def));
            first_element = false;
        }

        // Events
        for event_def in &contract.contents.events {
            if !first_element {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_event_ir(event_def));
            first_element = false;
        }

        // State variables - skip constants that were already emitted with structs
        for var_def in &contract.contents.variables {
            // Check if this is a constant that was already emitted
            let var_name = var_def.element.name.as_ref().map(|n| n.name.clone());
            if let Some(name) = &var_name {
                if emitted_constants.contains(name) {
                    continue; // Skip - already emitted before struct
                }
            }

            if !first_element {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_variable_definition_ir(var_def));
            first_element = false;
        }

        // Functions
        for func_def in &contract.contents.functions {
            if !first_element {
                body_ir.push(IRElement::HardLineBreak);
            }
            body_ir.push(IRElement::HardLineBreak);
            body_ir.extend(build_function_ir(func_def));
            first_element = false;
        }

        // Wrap entire body in single indent
        ir.push(IRElement::indent(body_ir));
        ir.push(IRElement::HardLineBreak);
    } else {
        // Empty contract body - add space before closing brace
        ir.push(IRElement::text(" "));
    }

    ir.push(IRElement::text("}"));
    ir
}

fn format_base(base: &Base) -> IRElement {
    let mut ir = vec![format_identifier_path(&base.name)];

    if let Some(args) = &base.args {
        ir.push(IRElement::text("("));
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                ir.push(IRElement::text(", "));
            }
            ir.push(format_expression(arg));
        }
        ir.push(IRElement::text(")"));
    }

    IRElement::group(ir)
}

fn build_event_ir(event: &CollectedEvent) -> Vec<IRElement> {
    let mut ir = vec![];

    // Check if event has any documentation
    let has_docs = !event.definition.leading_comments.is_empty()
        || !event.definition.trailing_comments.is_empty();

    if has_docs {
        let comments = &event.definition.leading_comments;

        if !comments.is_empty() {
            // Build structured NatSpec block for events, filling in missing params
            ir.extend(build_event_natspec_from_comments(
                comments,
                &event.parameters,
            ));
            ir.push(IRElement::HardLineBreak);
        }
    } else {
        // Generate NatSpec documentation for undocumented events
        ir.extend(generate_event_natspec(event));
        ir.push(IRElement::HardLineBreak);
    }

    // Build the event definition
    let event_ir = build_event_definition(event);
    ir.extend(event_ir);

    // Move trailing comments to the next line as leading comments
    if !event.definition.trailing_comments.is_empty() {
        ir.push(IRElement::HardLineBreak);
        for comment in &event.definition.trailing_comments {
            ir.push(build_comment(comment));
            ir.push(IRElement::HardLineBreak);
        }
    }

    ir
}

/// Build structured NatSpec IR from event comments, adding TODO for undocumented params
fn build_event_natspec_from_comments(
    comments: &[Comment],
    parameters: &[CommentedElement<Box<EventParameter>>],
) -> Vec<IRElement> {
    let mut description_parts = Vec::new();
    let mut param_docs = std::collections::HashMap::new();
    // Track which section we're in for multi-line continuations
    let mut current_param: Option<String> = None;

    // Parse all comment lines
    for comment in comments {
        let content = match comment {
            Comment::DocLine(_, text) => text
                .trim()
                .strip_prefix("///")
                .unwrap_or(text.trim())
                .trim(),
            Comment::DocBlock(_, text) => {
                let text = text.trim().strip_prefix("/**").unwrap_or(text.trim());
                text.strip_suffix("*/").unwrap_or(text).trim()
            }
            _ => continue,
        };

        // Handle multi-line block comments
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("@notice ") {
                current_param = None;
                description_parts.push(trimmed.strip_prefix("@notice ").unwrap().to_string());
            } else if trimmed.starts_with("@param ") {
                // Parse @param name description
                let rest = trimmed.strip_prefix("@param ").unwrap();
                if let Some((name, desc)) = rest.split_once(' ') {
                    param_docs.insert(name.to_string(), desc.to_string());
                    current_param = Some(name.to_string());
                } else {
                    param_docs.insert(rest.to_string(), "TODO".to_string());
                    current_param = Some(rest.to_string());
                }
            } else if !trimmed.is_empty() && !trimmed.starts_with("@") {
                // Continuation of previous section
                if let Some(ref param_name) = current_param {
                    // Continue previous @param
                    if let Some(desc) = param_docs.get_mut(param_name) {
                        desc.push(' ');
                        desc.push_str(trimmed);
                    }
                } else if !description_parts.is_empty() {
                    // Continue description
                    let last = description_parts.last_mut().unwrap();
                    last.push(' ');
                    last.push_str(trimmed);
                } else {
                    description_parts.push(trimmed.to_string());
                }
            }
        }
    }

    // Build the IR
    let mut ir = vec![IRElement::text("/**")];
    let mut content = vec![IRElement::HardLineBreak];

    // Add description
    let description = description_parts.join(" ");
    if !description.is_empty() {
        content.push(IRElement::group(text_with_word_breaks(&description)));
    } else {
        content.push(IRElement::text("TODO"));
    }

    // Add param lines - use event parameters to ensure all are documented
    if !parameters.is_empty() {
        content.push(IRElement::HardLineBreak);
        content.push(IRElement::HardLineBreak);

        for param in parameters {
            if let Some(param_name) = &param.element.name {
                let name = &param_name.name;
                let desc = param_docs.get(name).map(|s| s.as_str()).unwrap_or("TODO");
                // Build @param with word wrapping - prefix + description
                // Continuation lines get extra indent
                let desc_ir = text_with_word_breaks(desc);
                let mut param_ir = vec![IRElement::text(&format!("@param {} ", name))];
                param_ir.push(IRElement::indent(vec![IRElement::group(desc_ir)]));
                content.push(IRElement::group(param_ir));
                content.push(IRElement::HardLineBreak);
            }
        }

        // Remove last HardLineBreak
        content.pop();
    }

    ir.push(IRElement::indent(content));
    ir.push(IRElement::HardLineBreak);
    ir.push(IRElement::text("*/"));

    ir
}

fn build_event_definition(event: &CollectedEvent) -> Vec<IRElement> {
    let mut ir = vec![
        IRElement::text("event"),
        IRElement::text(" "),
        IRElement::text(&event.definition.element.name.as_ref().unwrap().name),
        IRElement::text(" ("),
    ];

    // Format event parameters - always on separate lines
    if !event.parameters.is_empty() {
        let mut params_ir = vec![];

        for (i, field) in event.parameters.iter().enumerate() {
            params_ir.push(IRElement::HardLineBreak);

            // Add type
            params_ir.push(format_expression(&field.element.ty));

            // Add indexed if present
            if field.element.indexed {
                params_ir.push(IRElement::text(" indexed"));
            }

            // Add name if present
            if let Some(name) = &field.element.name {
                params_ir.push(IRElement::text(" "));
                params_ir.push(IRElement::text(&name.name));
            }

            // Add comma except for last
            if i < event.parameters.len() - 1 {
                params_ir.push(IRElement::text(","));
            }
        }

        ir.push(IRElement::indent(params_ir));
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text(");"));

    if event.definition.element.anonymous {
        // Insert "anonymous" before the semicolon
        let last = ir.len() - 1;
        ir.insert(last, IRElement::text(" anonymous"));
    }

    vec![IRElement::group(ir)]
}

/// Generate NatSpec documentation for an undocumented event
fn generate_event_natspec(event: &CollectedEvent) -> Vec<IRElement> {
    let mut ir = vec![];

    ir.push(IRElement::text("/**"));

    // HardLineBreak at start ensures first line gets indented
    let mut content = vec![IRElement::HardLineBreak];

    // Add TODO description
    content.push(IRElement::text("TODO"));

    // Add documentation for event parameters if any
    if !event.parameters.is_empty() {
        content.push(IRElement::HardLineBreak);
        content.push(IRElement::HardLineBreak);

        for param in &event.parameters {
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

fn has_contract_contents(contents: &CollectedContractElements) -> bool {
    !contents.using_directives.is_empty()
        || !contents.types.is_empty()
        || !contents.enums.is_empty()
        || !contents.structs.is_empty()
        || !contents.errors.is_empty()
        || !contents.events.is_empty()
        || !contents.variables.is_empty()
        || !contents.functions.is_empty()
}
