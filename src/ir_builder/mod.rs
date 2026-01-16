//! IR Builder for CollectedElements
//!
//! This module transforms the collected AST elements with their associated comments
//! into an intermediate representation (IR) that can be used by the printer for
//! context-aware formatting and line-wrapping.
//!
//! The IR builder is responsible for:
//! - Converting CollectedElements into IRElements
//! - Properly placing comments (leading and trailing)
//! - Creating appropriate grouping and indentation structures
//! - Handling spacing between elements

pub mod ir;

mod comments;
mod contracts;
mod expressions;
mod functions;
mod licensing;
mod ordering;
mod statements;
mod toplevel;
mod types;

#[cfg(test)]
pub mod tests;

use crate::collector::model::CollectedElements;
use solang_parser::pt::Import;

use comments::*;
use contracts::*;
use functions::*;
use ir::IRElement;
use licensing::generate_spdx_header;
use ordering::order_collected_elements;
use toplevel::*;
use types::*;

/// Convert a string into Text elements with SoftLineBreak between each word
pub fn text_with_word_breaks(s: &str) -> Vec<IRElement> {
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut result = Vec::new();

    for (i, word) in words.iter().enumerate() {
        if i > 0 {
            result.push(IRElement::SoftLineBreak);
        }
        result.push(IRElement::text(*word));
    }

    result
}

/// Build IR from collected source unit elements
pub fn build_ir(collected: &CollectedElements) -> Vec<IRElement> {
    // Set up source for number literal preservation
    expressions::set_source_for_formatting(&collected.source);

    // First, reorder the elements according to our rules
    let ordered = order_collected_elements(collected);

    let mut ir = Vec::new();

    // Collect all comments from the source for SPDX header generation
    let mut all_comments = Vec::new();

    // Add standalone comments
    all_comments.extend(ordered.standalone_comments.clone());

    // Add comments from all commented elements
    for pragma in &ordered.pragmas {
        all_comments.extend(pragma.leading_comments.clone());
        all_comments.extend(pragma.trailing_comments.clone());
    }
    for import in &ordered.imports {
        all_comments.extend(import.leading_comments.clone());
        all_comments.extend(import.trailing_comments.clone());
    }
    for using in &ordered.using_directives {
        all_comments.extend(using.leading_comments.clone());
        all_comments.extend(using.trailing_comments.clone());
    }
    for error in &ordered.errors {
        all_comments.extend(error.definition.leading_comments.clone());
        all_comments.extend(error.definition.trailing_comments.clone());
    }
    for type_def in &ordered.types {
        all_comments.extend(type_def.leading_comments.clone());
        all_comments.extend(type_def.trailing_comments.clone());
    }
    for enum_def in &ordered.enums {
        all_comments.extend(enum_def.definition.leading_comments.clone());
        all_comments.extend(enum_def.definition.trailing_comments.clone());
    }
    for struct_def in &ordered.structs {
        all_comments.extend(struct_def.definition.leading_comments.clone());
        all_comments.extend(struct_def.definition.trailing_comments.clone());
    }
    for var in &ordered.variables {
        all_comments.extend(var.leading_comments.clone());
        all_comments.extend(var.trailing_comments.clone());
    }
    for func in &ordered.functions {
        all_comments.extend(func.definition.leading_comments.clone());
        all_comments.extend(func.definition.trailing_comments.clone());
    }
    for contract in &ordered.contracts {
        all_comments.extend(contract.definition.leading_comments.clone());
        all_comments.extend(contract.definition.trailing_comments.clone());
    }

    // Generate and add SPDX header as the first element
    let spdx_header = generate_spdx_header(&all_comments);
    ir.push(IRElement::text(spdx_header));

    // Add a single line break after SPDX header if there are other elements
    if has_other_elements(&ordered) || !ordered.pragmas.is_empty() {
        ir.push(IRElement::HardLineBreak);
    }

    // Process pragmas
    for (i, pragma) in ordered.pragmas.iter().enumerate() {
        if i > 0 {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_pragma_ir(pragma));
    }

    // Add blank line after pragmas if there are any and there are other elements
    if !ordered.pragmas.is_empty() && has_other_elements(&ordered) {
        ir.push(IRElement::HardLineBreak);
        ir.push(IRElement::HardLineBreak);
    }

    // Process imports - only add blank lines between groups (external, interfaces, local)
    let mut prev_group: Option<ImportGroup> = None;
    for import in ordered.imports.iter() {
        let current_group = get_import_group(&import.element);

        if let Some(prev) = prev_group {
            if prev != current_group {
                // Blank line between different groups
                ir.push(IRElement::HardLineBreak);
                ir.push(IRElement::HardLineBreak);
            } else {
                // Just a newline within the same group
                ir.push(IRElement::HardLineBreak);
            }
        }

        ir.extend(build_import_ir(import));
        prev_group = Some(current_group);
    }

    // Add blank line after imports if there are any and there are other elements after them
    if !ordered.imports.is_empty() && has_elements_after_imports(&ordered) {
        ir.push(IRElement::HardLineBreak);
        ir.push(IRElement::HardLineBreak);
    }

    // Process using directives
    for (i, using) in ordered.using_directives.iter().enumerate() {
        if i > 0 || !ordered.pragmas.is_empty() || !ordered.imports.is_empty() {
            if i == 0 {
                // First using directive after other elements
                ir.push(IRElement::HardLineBreak);
            } else {
                ir.push(IRElement::HardLineBreak);
            }
        }
        ir.extend(build_using_directive_ir(using));
    }

    // Add blank line after using directives
    if !ordered.using_directives.is_empty() && has_elements_after_using(&ordered) {
        ir.push(IRElement::HardLineBreak);
        ir.push(IRElement::HardLineBreak);
    }

    // Process type definitions
    for (i, type_def) in ordered.types.iter().enumerate() {
        if needs_preceding_blank_line(&ordered, ElementType::Type, i) {
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::HardLineBreak);
        } else if needs_preceding_newline(&ordered, ElementType::Type, i) {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_type_definition_ir(type_def));
    }

    // Process enums
    for (i, enum_def) in ordered.enums.iter().enumerate() {
        if needs_preceding_blank_line(&ordered, ElementType::Enum, i) {
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::HardLineBreak);
        } else if needs_preceding_newline(&ordered, ElementType::Enum, i) {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_enum_ir(enum_def));
    }

    // Process structs
    for (i, struct_def) in ordered.structs.iter().enumerate() {
        if needs_preceding_blank_line(&ordered, ElementType::Struct, i) {
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::HardLineBreak);
        } else if needs_preceding_newline(&ordered, ElementType::Struct, i) {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_struct_ir(struct_def));
    }

    // Process errors
    for (i, error_def) in ordered.errors.iter().enumerate() {
        if needs_preceding_blank_line(&ordered, ElementType::Error, i) {
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::HardLineBreak);
        } else if needs_preceding_newline(&ordered, ElementType::Error, i) {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_error_ir(error_def));
    }

    // Process variables
    for (i, var_def) in ordered.variables.iter().enumerate() {
        if needs_preceding_blank_line(&ordered, ElementType::Variable, i) {
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::HardLineBreak);
        } else if needs_preceding_newline(&ordered, ElementType::Variable, i) {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_variable_definition_ir(var_def));
    }

    // Process functions
    for (i, func_def) in ordered.functions.iter().enumerate() {
        if needs_preceding_blank_line(&ordered, ElementType::Function, i) {
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::HardLineBreak);
        } else if needs_preceding_newline(&ordered, ElementType::Function, i) {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_function_ir(func_def));
    }

    // Process contracts
    for (i, contract) in ordered.contracts.iter().enumerate() {
        if needs_preceding_blank_line(&ordered, ElementType::Contract, i) {
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::HardLineBreak);
        } else if needs_preceding_newline(&ordered, ElementType::Contract, i) {
            ir.push(IRElement::HardLineBreak);
        }
        ir.extend(build_contract_ir(contract));
    }

    // Process standalone comments at the end (filtering out SPDX and import section comments)
    for comment in &ordered.standalone_comments {
        if !is_spdx_comment(comment) && !is_import_section_comment(comment) {
            ir.push(IRElement::HardLineBreak);
            ir.push(build_comment(comment));
        }
    }

    // Clear the source after building IR
    expressions::clear_source_for_formatting();

    ir
}

#[derive(Debug, Clone, Copy)]
enum ElementType {
    Type,
    Enum,
    Struct,
    Error,
    Variable,
    Function,
    Contract,
}

/// Import grouping for spacing
#[derive(Debug, Clone, Copy, PartialEq)]
enum ImportGroup {
    External,   // Starts with '@'
    Interfaces, // Starts with "interfaces"
    Local,      // Everything else
}

/// Determine which group an import belongs to
fn get_import_group(import: &Import) -> ImportGroup {
    let path = ordering::get_import_path_string(import);

    if path.starts_with('@') {
        ImportGroup::External
    } else if path.starts_with("interfaces") {
        ImportGroup::Interfaces
    } else {
        ImportGroup::Local
    }
}

/// Check if we need a blank line before an element
fn needs_preceding_blank_line(
    ordered: &CollectedElements,
    element_type: ElementType,
    index: usize,
) -> bool {
    // First element of its type after imports/using/pragmas needs blank line
    if index == 0 {
        match element_type {
            ElementType::Type => {
                !ordered.pragmas.is_empty()
                    || !ordered.imports.is_empty()
                    || !ordered.using_directives.is_empty()
            }
            ElementType::Enum => {
                // Don't add newline after pragmas/imports/using - they already add trailing blank lines
                !ordered.types.is_empty()
            }
            ElementType::Struct => {
                // Don't add newline after pragmas/imports/using - they already add trailing blank lines
                !ordered.types.is_empty() || !ordered.enums.is_empty()
            }
            ElementType::Error => {
                // Don't add newline after pragmas/imports/using - they already add trailing blank lines
                !ordered.types.is_empty()
                    || !ordered.enums.is_empty()
                    || !ordered.structs.is_empty()
            }
            ElementType::Variable => {
                // Don't add newline after pragmas/imports/using - they already add trailing blank lines
                !ordered.types.is_empty()
                    || !ordered.enums.is_empty()
                    || !ordered.structs.is_empty()
                    || !ordered.errors.is_empty()
            }
            ElementType::Function => {
                // Don't add newline after pragmas/imports/using - they already add trailing blank lines
                !ordered.types.is_empty()
                    || !ordered.enums.is_empty()
                    || !ordered.structs.is_empty()
                    || !ordered.errors.is_empty()
                    || !ordered.variables.is_empty()
            }
            ElementType::Contract => {
                // Don't add blank line if only pragmas/imports/using before us
                // (they already add their own blank lines)
                !ordered.types.is_empty()
                    || !ordered.enums.is_empty()
                    || !ordered.structs.is_empty()
                    || !ordered.errors.is_empty()
                    || !ordered.variables.is_empty()
                    || !ordered.functions.is_empty()
            }
        }
    } else {
        // Blank line between elements of the same type
        true
    }
}

/// Check if we need a newline before an element
fn needs_preceding_newline(
    ordered: &CollectedElements,
    element_type: ElementType,
    index: usize,
) -> bool {
    // If we don't need a blank line but this isn't the first element overall, we need a newline
    !needs_preceding_blank_line(ordered, element_type, index)
        && has_any_preceding_element(ordered, element_type, index)
}

// Helper functions to check for elements
fn has_other_elements(ordered: &CollectedElements) -> bool {
    !ordered.imports.is_empty()
        || !ordered.using_directives.is_empty()
        || !ordered.types.is_empty()
        || !ordered.enums.is_empty()
        || !ordered.structs.is_empty()
        || !ordered.errors.is_empty()
        || !ordered.variables.is_empty()
        || !ordered.functions.is_empty()
        || !ordered.contracts.is_empty()
}

fn has_elements_after_imports(ordered: &CollectedElements) -> bool {
    !ordered.using_directives.is_empty()
        || !ordered.types.is_empty()
        || !ordered.enums.is_empty()
        || !ordered.structs.is_empty()
        || !ordered.errors.is_empty()
        || !ordered.variables.is_empty()
        || !ordered.functions.is_empty()
        || !ordered.contracts.is_empty()
}

fn has_elements_after_using(ordered: &CollectedElements) -> bool {
    !ordered.types.is_empty()
        || !ordered.enums.is_empty()
        || !ordered.structs.is_empty()
        || !ordered.errors.is_empty()
        || !ordered.variables.is_empty()
        || !ordered.functions.is_empty()
        || !ordered.contracts.is_empty()
}

fn has_preceding_elements_struct(ordered: &CollectedElements) -> bool {
    !ordered.pragmas.is_empty()
        || !ordered.imports.is_empty()
        || !ordered.using_directives.is_empty()
        || !ordered.types.is_empty()
        || !ordered.enums.is_empty()
}

fn has_preceding_elements_error(ordered: &CollectedElements) -> bool {
    !ordered.pragmas.is_empty()
        || !ordered.imports.is_empty()
        || !ordered.using_directives.is_empty()
        || !ordered.types.is_empty()
        || !ordered.enums.is_empty()
        || !ordered.structs.is_empty()
}

fn has_preceding_elements_variable(ordered: &CollectedElements) -> bool {
    !ordered.pragmas.is_empty()
        || !ordered.imports.is_empty()
        || !ordered.using_directives.is_empty()
        || !ordered.types.is_empty()
        || !ordered.enums.is_empty()
        || !ordered.structs.is_empty()
        || !ordered.errors.is_empty()
}

fn has_preceding_elements_function(ordered: &CollectedElements) -> bool {
    !ordered.pragmas.is_empty()
        || !ordered.imports.is_empty()
        || !ordered.using_directives.is_empty()
        || !ordered.types.is_empty()
        || !ordered.enums.is_empty()
        || !ordered.structs.is_empty()
        || !ordered.errors.is_empty()
        || !ordered.variables.is_empty()
}

fn has_any_preceding_element(
    ordered: &CollectedElements,
    element_type: ElementType,
    index: usize,
) -> bool {
    match element_type {
        ElementType::Type => {
            index > 0
                || !ordered.pragmas.is_empty()
                || !ordered.imports.is_empty()
                || !ordered.using_directives.is_empty()
        }
        ElementType::Enum => {
            // Only add newline if there are types before (not just pragmas/imports/using)
            // Pragmas, imports, using already provide their own trailing newlines
            index > 0 || !ordered.types.is_empty()
        }
        ElementType::Struct => {
            // Only add newline if there are types/enums before
            index > 0 || !ordered.types.is_empty() || !ordered.enums.is_empty()
        }
        ElementType::Error => {
            // Only add newline if there are types/enums/structs before
            index > 0
                || !ordered.types.is_empty()
                || !ordered.enums.is_empty()
                || !ordered.structs.is_empty()
        }
        ElementType::Variable => {
            // Only add newline if there are types/enums/structs/errors before
            index > 0
                || !ordered.types.is_empty()
                || !ordered.enums.is_empty()
                || !ordered.structs.is_empty()
                || !ordered.errors.is_empty()
        }
        ElementType::Function => {
            // Only add newline if there are types/enums/structs/errors/variables before
            index > 0
                || !ordered.types.is_empty()
                || !ordered.enums.is_empty()
                || !ordered.structs.is_empty()
                || !ordered.errors.is_empty()
                || !ordered.variables.is_empty()
        }
        ElementType::Contract => {
            // For contracts, only add newline if there are elements other than pragma/import/using
            // (those already add their own trailing blank lines)
            index > 0
                || !ordered.types.is_empty()
                || !ordered.enums.is_empty()
                || !ordered.structs.is_empty()
                || !ordered.errors.is_empty()
                || !ordered.variables.is_empty()
                || !ordered.functions.is_empty()
        }
    }
}
