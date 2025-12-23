//! Peck transformation module for extracting public interfaces from Solidity contracts.
//!
//! This module transforms `CollectedElements` to represent the public interface
//! of a contract, suitable for generating Solidity interface files.

use crate::collector::model::*;
use solang_parser::pt::*;

/// Transform collected elements into an interface representation.
///
/// This transformation:
/// - Changes `contract X` to `interface IX`
/// - Transforms `@title X` to `@title X Interface`
/// - Converts `@custom:param` to `@param` and `@custom:return` to `@return`
/// - Filters out events and modifiers
/// - Converts public constants to getter function signatures
/// - Converts public mappings to getter function signatures
/// - Converts public/external functions to external signatures without bodies
pub fn transform_to_interface(collected: &CollectedElements) -> CollectedElements {
    let mut result = CollectedElements::new();

    // Copy pragmas as-is
    result.pragmas = collected.pragmas.clone();

    // Copy imports as-is (interfaces may need the same imports)
    result.imports = collected.imports.clone();

    // Transform contracts to interfaces
    for contract in &collected.contracts {
        result.contracts.push(transform_contract(contract));
    }

    result
}

/// Transform a contract into an interface.
fn transform_contract(contract: &CollectedContract) -> CollectedContract {
    let mut definition = contract.definition.clone();

    // Change contract type to interface
    let interface_loc = match &definition.element.ty {
        ContractTy::Contract(loc) => *loc,
        ContractTy::Abstract(loc) => *loc,
        _ => definition.element.loc,
    };

    // Create a mutable copy of the contract definition
    let mut new_def = (*definition.element).clone();
    new_def.ty = ContractTy::Interface(interface_loc);

    // Prepend "I" to the contract name
    if let Some(ref name) = new_def.name {
        let mut new_name = name.clone();
        new_name.name = format!("I{}", name.name);
        new_def.name = Some(new_name);
    }

    // Clear base contracts (interfaces don't inherit from contracts)
    new_def.base = vec![];

    definition.element = Box::new(new_def);

    // Transform leading comments (NatSpec)
    definition.leading_comments = transform_contract_comments(&definition.leading_comments);

    // Transform contract contents
    let contents = transform_contract_contents(&contract.contents);

    CollectedContract {
        definition,
        contents,
    }
}

/// Transform contract-level NatSpec comments.
/// - `@title X` becomes `@title X Interface`
fn transform_contract_comments(comments: &[Comment]) -> Vec<Comment> {
    comments
        .iter()
        .map(|comment| match comment {
            Comment::DocBlock(loc, content) => {
                Comment::DocBlock(*loc, transform_title_in_natspec(content))
            }
            Comment::DocLine(loc, content) => {
                Comment::DocLine(*loc, transform_title_in_natspec(content))
            }
            other => other.clone(),
        })
        .collect()
}

/// Transform @title tag to append "Interface".
fn transform_title_in_natspec(content: &str) -> String {
    // Handle @title tag - append "Interface"
    let mut result = String::new();
    let mut in_title = false;

    for line in content.lines() {
        if !result.is_empty() {
            result.push('\n');
        }

        let trimmed = line.trim().trim_start_matches('*').trim();

        if trimmed.starts_with("@title ") {
            // Found @title - append "Interface" to the title
            let title_content = trimmed.strip_prefix("@title ").unwrap().trim();
            // Preserve leading whitespace/stars from original line
            let prefix = if line.contains('*') {
                let star_pos = line.find('*').unwrap();
                &line[..=star_pos]
            } else {
                ""
            };
            if prefix.is_empty() {
                result.push_str(&format!("@title {} Interface", title_content));
            } else {
                result.push_str(&format!("{} @title {} Interface", prefix, title_content));
            }
            in_title = true;
        } else if in_title && !trimmed.is_empty() && !trimmed.starts_with("@") {
            // Continuation of title - just append as-is, Interface already added
            result.push_str(line);
            in_title = false;
        } else {
            result.push_str(line);
            if trimmed.starts_with("@") {
                in_title = false;
            }
        }
    }

    result
}

/// Transform contract contents for interface extraction.
fn transform_contract_contents(contents: &CollectedContractElements) -> CollectedContractElements {
    let mut result = CollectedContractElements::new();

    // Keep structs (interfaces can have struct definitions)
    result.structs = contents.structs.clone();

    // Keep enums (interfaces can have enum definitions)
    result.enums = contents.enums.clone();

    // Keep errors (interfaces can have error definitions)
    result.errors = contents.errors.clone();

    // Keep type definitions
    result.types = contents.types.clone();

    // Filter out events - interfaces don't include events
    // (events are implementation details)

    // Convert public constants to getter functions
    for var in &contents.variables {
        if is_public_variable(&var.element) {
            if let Some(func) = variable_to_getter_function(var) {
                result.functions.push(func);
            }
        }
    }

    // Transform functions - keep only public/external, strip bodies
    for func in &contents.functions {
        if let Some(transformed) = transform_function_for_interface(func) {
            result.functions.push(transformed);
        }
    }

    result
}

/// Check if a variable is public.
fn is_public_variable(var: &VariableDefinition) -> bool {
    var.attrs.iter().any(|attr| {
        matches!(
            attr,
            VariableAttribute::Visibility(Visibility::Public(_))
                | VariableAttribute::Visibility(Visibility::External(_))
        )
    })
}

/// Convert a public variable to a getter function.
fn variable_to_getter_function(
    var: &CommentedElement<Box<VariableDefinition>>,
) -> Option<CollectedFunction> {
    let var_def = &var.element;
    let var_name = var_def.name.as_ref()?.name.clone();

    // Transform comments: @custom:param -> @param, @custom:return -> @return
    let leading_comments = transform_variable_comments(&var.leading_comments);

    // Determine if this is a constant (pure) or a state variable (view)
    let is_constant = var_def.attrs.iter().any(|attr| {
        matches!(attr, VariableAttribute::Constant(_) | VariableAttribute::Immutable(_))
    });

    // Build function parameters and return type based on variable type
    let (params, returns, mutability) = build_getter_signature(&var_def.ty, is_constant);

    // Create function definition
    let func_def = FunctionDefinition {
        loc: var_def.loc,
        loc_prototype: var_def.loc,
        ty: FunctionTy::Function,
        name: Some(Identifier {
            loc: var_def.name.as_ref()?.loc,
            name: var_name,
        }),
        name_loc: var_def.name.as_ref()?.loc,
        params,
        attributes: build_getter_attributes(mutability),
        return_not_returns: None,
        returns,
        body: None, // No body for interface functions
    };

    Some(CollectedFunction {
        definition: CommentedElement {
            element: Box::new(func_def),
            leading_comments,
            trailing_comments: vec![],
            type_comments: None,
        },
        body_statements: vec![],
        body_standalone_comments: vec![],
        signature_comments: None,
    })
}

/// Build getter function signature from variable type.
/// Returns (params, returns, mutability).
fn build_getter_signature(
    ty: &Expression,
    is_constant: bool,
) -> (Vec<(Loc, Option<Parameter>)>, Vec<(Loc, Option<Parameter>)>, Mutability) {
    match ty {
        Expression::Type(loc, Type::Mapping { key, key_name, value, .. }) => {
            // Mapping getter: takes key as parameter, returns value
            let mut params = vec![];
            let mut current_value = value.as_ref();

            // First key parameter
            let key_param = Parameter {
                loc: *loc,
                ty: key.as_ref().clone(),
                storage: None,
                name: key_name.clone(),
                annotation: None,
            };
            params.push((*loc, Some(key_param)));

            // Handle nested mappings
            while let Expression::Type(inner_loc, Type::Mapping {
                key: inner_key,
                key_name: inner_key_name,
                value: inner_value,
                ..
            }) = current_value {
                let nested_key_param = Parameter {
                    loc: *inner_loc,
                    ty: inner_key.as_ref().clone(),
                    storage: None,
                    name: inner_key_name.clone(),
                    annotation: None,
                };
                params.push((*inner_loc, Some(nested_key_param)));
                current_value = inner_value.as_ref();
            }

            // Return type is the final value type
            // Reference types (string, bytes, arrays, structs) need memory storage
            let storage = get_return_storage_for_type(current_value, *loc);
            let return_param = Parameter {
                loc: *loc,
                ty: current_value.clone(),
                storage,
                name: None,
                annotation: None,
            };
            let returns = vec![(*loc, Some(return_param))];

            // Mappings are view (state access)
            (params, returns, Mutability::View(*loc))
        }
        Expression::Type(loc, Type::DynamicBytes) => {
            // bytes storage variable - returns bytes memory
            let return_param = Parameter {
                loc: *loc,
                ty: ty.clone(),
                storage: Some(StorageLocation::Memory(*loc)),
                name: None,
                annotation: None,
            };
            let returns = vec![(*loc, Some(return_param))];
            let mutability = if is_constant {
                Mutability::Pure(*loc)
            } else {
                Mutability::View(*loc)
            };
            (vec![], returns, mutability)
        }
        Expression::Type(loc, Type::String) => {
            // string storage variable - returns string memory
            let return_param = Parameter {
                loc: *loc,
                ty: ty.clone(),
                storage: Some(StorageLocation::Memory(*loc)),
                name: None,
                annotation: None,
            };
            let returns = vec![(*loc, Some(return_param))];
            let mutability = if is_constant {
                Mutability::Pure(*loc)
            } else {
                Mutability::View(*loc)
            };
            (vec![], returns, mutability)
        }
        // Contract/interface types become address
        Expression::Variable(_) => {
            // User-defined type (contract, interface, etc.) - convert to address
            let loc = get_expression_loc(ty);
            let address_ty = Expression::Type(loc, Type::Address);
            let return_param = Parameter {
                loc,
                ty: address_ty,
                storage: None,
                name: None,
                annotation: None,
            };
            let returns = vec![(loc, Some(return_param))];
            let mutability = if is_constant {
                Mutability::Pure(loc)
            } else {
                Mutability::View(loc)
            };
            (vec![], returns, mutability)
        }
        _ => {
            // Simple type - just return it
            let loc = get_expression_loc(ty);
            let return_param = Parameter {
                loc,
                ty: ty.clone(),
                storage: None,
                name: None,
                annotation: None,
            };
            let returns = vec![(loc, Some(return_param))];
            let mutability = if is_constant {
                Mutability::Pure(loc)
            } else {
                Mutability::View(loc)
            };
            (vec![], returns, mutability)
        }
    }
}

/// Get location from an expression.
fn get_expression_loc(expr: &Expression) -> Loc {
    match expr {
        Expression::Type(loc, _) => *loc,
        Expression::Variable(id) => id.loc,
        _ => Loc::Builtin,
    }
}

/// Determine if a type needs memory storage location in return position.
/// Reference types (string, bytes, arrays, structs) need memory.
fn get_return_storage_for_type(ty: &Expression, loc: Loc) -> Option<StorageLocation> {
    match ty {
        Expression::Type(_, Type::String) => Some(StorageLocation::Memory(loc)),
        Expression::Type(_, Type::DynamicBytes) => Some(StorageLocation::Memory(loc)),
        // Arrays need memory
        Expression::ArraySubscript(_, _, _) => Some(StorageLocation::Memory(loc)),
        // User-defined types (structs) need memory
        Expression::Variable(_) => Some(StorageLocation::Memory(loc)),
        _ => None,
    }
}

/// Build function attributes for a getter (external + mutability).
fn build_getter_attributes(mutability: Mutability) -> Vec<FunctionAttribute> {
    vec![
        FunctionAttribute::Visibility(Visibility::External(Some(Loc::Builtin))),
        FunctionAttribute::Mutability(mutability),
    ]
}

/// Transform variable comments: @custom:param -> @param, @custom:return -> @return.
fn transform_variable_comments(comments: &[Comment]) -> Vec<Comment> {
    comments
        .iter()
        .map(|comment| match comment {
            Comment::DocBlock(loc, content) => {
                Comment::DocBlock(*loc, transform_custom_tags(content))
            }
            Comment::DocLine(loc, content) => {
                Comment::DocLine(*loc, transform_custom_tags(content))
            }
            other => other.clone(),
        })
        .collect()
}

/// Transform @custom:param to @param and @custom:return to @return.
fn transform_custom_tags(content: &str) -> String {
    content
        .replace("@custom:param", "@param")
        .replace("@custom:return", "@return")
}

/// Transform a function for interface output.
/// Returns None if the function should be excluded (private/internal, modifier, etc.)
fn transform_function_for_interface(func: &CollectedFunction) -> Option<CollectedFunction> {
    let func_def = &func.definition.element;

    // Skip modifiers
    if matches!(func_def.ty, FunctionTy::Modifier) {
        return None;
    }

    // Skip constructors
    if matches!(func_def.ty, FunctionTy::Constructor) {
        return None;
    }

    // Check visibility - only public and external
    let is_public_or_external = func_def.attributes.iter().any(|attr| {
        matches!(
            attr,
            FunctionAttribute::Visibility(Visibility::Public(_))
                | FunctionAttribute::Visibility(Visibility::External(_))
        )
    });

    if !is_public_or_external {
        return None;
    }

    // Create new function definition without body and with external visibility
    let mut new_def = func_def.as_ref().clone();
    new_def.body = None; // Remove body

    // Transform attributes: change public to external, remove modifiers
    new_def.attributes = new_def
        .attributes
        .iter()
        .filter_map(|attr| match attr {
            FunctionAttribute::Visibility(Visibility::Public(loc)) => {
                Some(FunctionAttribute::Visibility(Visibility::External(*loc)))
            }
            FunctionAttribute::Visibility(Visibility::External(loc)) => {
                Some(FunctionAttribute::Visibility(Visibility::External(*loc)))
            }
            FunctionAttribute::Visibility(_) => None, // Remove internal/private
            FunctionAttribute::Mutability(m) => {
                Some(FunctionAttribute::Mutability(m.clone()))
            }
            FunctionAttribute::Virtual(_) => None, // Remove virtual
            FunctionAttribute::Override(_, _) => None, // Remove override
            FunctionAttribute::BaseOrModifier(_, _) => None, // Remove modifiers
            FunctionAttribute::Immutable(_) => None,
            FunctionAttribute::Error(_) => None,
        })
        .collect();

    Some(CollectedFunction {
        definition: CommentedElement {
            element: Box::new(new_def),
            leading_comments: func.definition.leading_comments.clone(),
            trailing_comments: vec![],
            type_comments: None,
        },
        body_statements: vec![],
        body_standalone_comments: vec![],
        signature_comments: func.signature_comments.clone(),
    })
}
