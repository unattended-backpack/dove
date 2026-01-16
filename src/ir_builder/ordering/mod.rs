//! Element Ordering Module
//!
//! This module handles the systematic organization of all Solidity source code
//! elements according to professional coding standards and best practices.
//!
//! Key responsibilities:
//! - Order all elements in CollectedElements according to established rules
//! - Ensure SPDX license header is always first (handled by IR builder)
//! - Order pragma directives by type and semantic version
//! - Group and sort imports by category (@external, interfaces, local)
//! - Order contracts by type (library, interface, abstract, concrete)
//! - Maintain proper element grouping and spacing

#[cfg(test)]
mod tests;

use crate::collector::model::{
    CollectedContract, CollectedElements, CommentedElement, GroupedCommentedImports,
};
use solang_parser::pt::*;

/// Order pragma directives according to professional standards
///
/// Orders pragmas by type with the following priority:
/// 1. Version pragmas (highest semantic version only)
/// 2. Identifier pragmas (sorted alphabetically by first identifier)
/// 3. String literal pragmas (sorted alphabetically by identifier)
///
/// # Arguments
///
/// * `pragmas` - Pragma directives with associated comments
///
/// # Returns
///
/// Ordered vector of pragma directives with original comments preserved
pub fn order_pragmas(
    pragmas: &[CommentedElement<Box<PragmaDirective>>],
) -> Vec<CommentedElement<Box<PragmaDirective>>> {
    let mut version_pragmas = Vec::new();
    let mut identifier_pragmas = Vec::new();
    let mut string_literal_pragmas = Vec::new();

    // Separate pragmas by type
    for commented_pragma in pragmas {
        match commented_pragma.element.as_ref() {
            PragmaDirective::Version(_, _, _) => version_pragmas.push(commented_pragma.clone()),
            PragmaDirective::Identifier(_, _, _) => {
                identifier_pragmas.push(commented_pragma.clone())
            }
            PragmaDirective::StringLiteral(_, _, _) => {
                string_literal_pragmas.push(commented_pragma.clone())
            }
        }
    }

    let mut result = Vec::new();

    // 1. Add highest version pragma only
    if let Some(highest_version) = find_highest_version_pragma(&version_pragmas) {
        result.push(highest_version);
    }

    // 2. Sort and add Identifier pragmas alphabetically by first_ident
    identifier_pragmas.sort_by(|a, b| {
        let name_a = get_identifier_pragma_first_name(&a.element);
        let name_b = get_identifier_pragma_first_name(&b.element);
        name_a.cmp(&name_b)
    });
    result.extend(identifier_pragmas);

    // 3. Sort and add StringLiteral pragmas alphabetically by ident
    string_literal_pragmas.sort_by(|a, b| {
        let name_a = get_string_literal_pragma_name(&a.element);
        let name_b = get_string_literal_pragma_name(&b.element);
        name_a.cmp(&name_b)
    });
    result.extend(string_literal_pragmas);

    result
}

/// Find the pragma directive with the highest semantic version
///
/// When multiple version pragmas exist, selects the one with the highest
/// semantic version number to ensure compatibility.
///
/// # Arguments
///
/// * `version_pragmas` - Vector of version pragma directives to compare
///
/// # Returns
///
/// The pragma directive with the highest semantic version, if any exist
pub fn find_highest_version_pragma(
    version_pragmas: &[CommentedElement<Box<PragmaDirective>>],
) -> Option<CommentedElement<Box<PragmaDirective>>> {
    if version_pragmas.is_empty() {
        return None;
    }

    // Find the pragma with the highest semantic version
    let mut highest_pragma = &version_pragmas[0];
    let mut highest_version = extract_version_numbers(&highest_pragma.element);

    for pragma in &version_pragmas[1..] {
        let current_version = extract_version_numbers(&pragma.element);
        if compare_versions(&current_version, &highest_version) > 0 {
            highest_pragma = pragma;
            highest_version = current_version;
        }
    }

    Some(highest_pragma.clone())
}

/// Extract semantic version numbers from a pragma directive
///
/// Parses version pragma directives to extract numeric version components
/// for comparison purposes.
///
/// # Arguments
///
/// * `pragma` - The pragma directive to parse
///
/// # Returns
///
/// Vector of version numbers (major, minor, patch, etc.)
pub fn extract_version_numbers(pragma: &Box<PragmaDirective>) -> Vec<u32> {
    match pragma.as_ref() {
        PragmaDirective::Version(_, _, version_reqs) => {
            // Extract version from the first version requirement
            if let Some(first_req) = version_reqs.first() {
                match first_req {
                    VersionComparator::Plain { version, .. }
                    | VersionComparator::Operator { version, .. } => version
                        .iter()
                        .filter_map(|v| v.parse::<u32>().ok())
                        .collect(),
                    _ => vec![0, 0, 0], // Default for complex version patterns
                }
            } else {
                vec![0, 0, 0]
            }
        }
        _ => vec![0, 0, 0],
    }
}

/// Compare two semantic versions using standard semver rules
///
/// Compares version arrays component by component to determine precedence.
///
/// # Arguments
///
/// * `a` - First version to compare
/// * `b` - Second version to compare
///
/// # Returns
///
/// - Positive if a > b
/// - Negative if a < b
/// - Zero if a == b
pub fn compare_versions(a: &[u32], b: &[u32]) -> i32 {
    let max_len = a.len().max(b.len());

    for i in 0..max_len {
        let a_val = a.get(i).unwrap_or(&0);
        let b_val = b.get(i).unwrap_or(&0);

        match a_val.cmp(b_val) {
            std::cmp::Ordering::Greater => return 1,
            std::cmp::Ordering::Less => return -1,
            std::cmp::Ordering::Equal => continue,
        }
    }

    0 // Versions are equal
}

/// Extract first identifier name from identifier pragma directive
///
/// # Arguments
///
/// * `pragma` - The pragma directive to parse
///
/// # Returns
///
/// The first identifier name, or empty string if not applicable
pub fn get_identifier_pragma_first_name(pragma: &Box<PragmaDirective>) -> String {
    match pragma.as_ref() {
        PragmaDirective::Identifier(_, first_ident, _) => first_ident
            .as_ref()
            .map(|i| i.name.clone())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

/// Extract identifier name from string literal pragma directive
///
/// # Arguments
///
/// * `pragma` - The pragma directive to parse
///
/// # Returns
///
/// The identifier name, or empty string if not applicable
pub fn get_string_literal_pragma_name(pragma: &Box<PragmaDirective>) -> String {
    match pragma.as_ref() {
        PragmaDirective::StringLiteral(_, ident, _) => ident.name.clone(),
        _ => String::new(),
    }
}

/// Extract import path string from import statement
///
/// Handles all import statement variants to extract the path for comparison.
///
/// # Arguments
///
/// * `import` - The import statement to process
///
/// # Returns
///
/// String representation of the import path
pub fn get_import_path_string(import: &Import) -> String {
    match import {
        Import::Plain(path, _) => format_import_path(path),
        Import::GlobalSymbol(path, _, _) => format_import_path(path),
        Import::Rename(path, _, _) => format_import_path(path),
    }
}

/// Order commented import statements preserving associated comments
///
/// Same grouping logic as order_imports() but preserves leading and trailing
/// comments associated with each import statement.
///
/// # Arguments
///
/// * `imports` - Import statements with associated comments
///
/// # Returns
///
/// Grouped and sorted commented imports organized by category
pub fn order_commented_imports(
    imports: &[CommentedElement<Box<Import>>],
) -> GroupedCommentedImports {
    let mut external = Vec::new();
    let mut interfaces = Vec::new();
    let mut local = Vec::new();

    for import in imports {
        let path = get_import_path_string(&import.element);

        if path.starts_with('@') {
            external.push(import.clone());
        } else if path.starts_with("interfaces") {
            interfaces.push(import.clone());
        } else {
            local.push(import.clone());
        }
    }

    // Sort each group alphabetically by path (case-insensitive)
    external.sort_by(|a, b| {
        get_import_path_string(&a.element)
            .to_lowercase()
            .cmp(&get_import_path_string(&b.element).to_lowercase())
    });
    interfaces.sort_by(|a, b| {
        get_import_path_string(&a.element)
            .to_lowercase()
            .cmp(&get_import_path_string(&b.element).to_lowercase())
    });
    local.sort_by(|a, b| {
        get_import_path_string(&a.element)
            .to_lowercase()
            .cmp(&get_import_path_string(&b.element).to_lowercase())
    });

    GroupedCommentedImports {
        external,
        interfaces,
        local,
    }
}

/// Format import path from ImportPath AST node
///
/// # Arguments
///
/// * `path` - The import path AST node
///
/// # Returns
///
/// String representation of the import path
pub fn format_import_path(path: &ImportPath) -> String {
    match path {
        ImportPath::Filename(string_lit) => string_lit.string.clone(),
        ImportPath::Path(path) => format_identifier_path(path),
    }
}

/// Format identifier path from IdentifierPath AST node
///
/// # Arguments
///
/// * `path` - The identifier path AST node
///
/// # Returns
///
/// String representation joining identifiers with dots
pub fn format_identifier_path(path: &IdentifierPath) -> String {
    path.identifiers
        .iter()
        .map(|i| i.name.as_str())
        .collect::<Vec<_>>()
        .join(".")
}

/// Order all elements in CollectedElements according to established rules
///
/// Applies comprehensive ordering to all source unit elements:
/// 1. Pragmas (ensuring SPDX is handled by IR builder)
/// 2. Imports (external, interfaces, local)
/// 3. Top-level errors
/// 4. Top-level type definitions
/// 5. Top-level using directives
/// 6. Top-level enums
/// 7. Top-level structs
/// 8. Top-level functions
/// 9. Top-level variables
/// 10. Contracts (library, interface, abstract, concrete)
///
/// # Arguments
///
/// * `elements` - The collected elements to reorder
///
/// # Returns
///
/// A new CollectedElements with all elements reordered
pub fn order_collected_elements(elements: &CollectedElements) -> CollectedElements {
    CollectedElements {
        // Preserve source for number literal formatting
        source: elements.source.clone(),

        // Order pragmas
        pragmas: order_pragmas(&elements.pragmas),

        // Order imports
        imports: order_imports_commented(&elements.imports),

        // Other top-level elements remain in their original order for now
        errors: elements.errors.clone(),
        types: elements.types.clone(),
        using_directives: elements.using_directives.clone(),
        enums: elements.enums.clone(),
        structs: elements.structs.clone(),
        functions: elements.functions.clone(),
        variables: elements.variables.clone(),

        // Order contracts by type
        contracts: order_contracts(&elements.contracts),

        // Standalone comments remain unchanged
        standalone_comments: elements.standalone_comments.clone(),
    }
}

/// Order contracts by type
///
/// Orders contracts with the following priority:
/// 1. Library contracts
/// 2. Interface contracts
/// 3. Abstract contracts
/// 4. Regular concrete contracts
///
/// Within each category, contracts maintain their original order.
///
/// # Arguments
///
/// * `contracts` - The contracts to order
///
/// # Returns
///
/// Ordered vector of contracts
pub fn order_contracts(contracts: &[CollectedContract]) -> Vec<CollectedContract> {
    let mut libraries = Vec::new();
    let mut interfaces = Vec::new();
    let mut abstracts = Vec::new();
    let mut concretes = Vec::new();

    for contract in contracts {
        match &contract.definition.element.ty {
            ContractTy::Library(_) => libraries.push(contract.clone()),
            ContractTy::Interface(_) => interfaces.push(contract.clone()),
            ContractTy::Abstract(_) => abstracts.push(contract.clone()),
            ContractTy::Contract(_) => concretes.push(contract.clone()),
        }
    }

    // Combine in the specified order
    let mut result = Vec::new();
    result.extend(libraries);
    result.extend(interfaces);
    result.extend(abstracts);
    result.extend(concretes);

    result
}

/// Order imports preserving comments
///
/// Converts the order_commented_imports result into a flat vector
/// maintaining the group order: external, interfaces, local
///
/// # Arguments
///
/// * `imports` - Import statements with comments to order
///
/// # Returns
///
/// Ordered vector of commented imports
pub fn order_imports_commented(
    imports: &[CommentedElement<Box<Import>>],
) -> Vec<CommentedElement<Box<Import>>> {
    let grouped = order_commented_imports(imports);

    let mut result = Vec::new();
    result.extend(grouped.external);
    result.extend(grouped.interfaces);
    result.extend(grouped.local);

    result
}
