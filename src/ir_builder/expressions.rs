//! Expression formatting for the IR builder

use super::ir::IRElement;
use solang_parser::pt::*;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// Thread-local storage for the source code during IR building.
    /// Used to preserve original number literal formatting (underscores).
    static SOURCE: RefCell<String> = RefCell::new(String::new());
}

/// Set the source code for number literal preservation during IR building.
pub fn set_source_for_formatting(source: &str) {
    SOURCE.with(|s| {
        *s.borrow_mut() = source.to_string();
    });
}

/// Clear the source code after IR building is complete.
pub fn clear_source_for_formatting() {
    SOURCE.with(|s| {
        s.borrow_mut().clear();
    });
}

/// Extract text from the source at the given location.
fn extract_source_text(loc: &Loc) -> Option<String> {
    if let Loc::File(_, start, end) = loc {
        SOURCE.with(|s| {
            let source = s.borrow();
            if source.is_empty() || *end > source.len() {
                None
            } else {
                Some(source[*start..*end].to_string())
            }
        })
    } else {
        None
    }
}

/// Normalize a parameter name to have a leading underscore
pub fn normalize_param_name(name: &str) -> String {
    if name.starts_with('_') {
        name.to_string()
    } else {
        format!("_{}", name)
    }
}

/// Transform a return variable name to an output variable name.
/// e.g., `config_` -> `_configOutput`, `value_` -> `_valueOutput`
pub fn transform_return_var_name(name: &str) -> String {
    let base = name.strip_suffix('_').unwrap_or(name);
    // Also normalize with underscore prefix
    normalize_param_name(&format!("{}Output", base))
}

/// Build a rename map for function parameters (original name -> normalized name)
pub fn build_param_rename_map(params: &[(Loc, Option<Parameter>)]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (_, param) in params {
        if let Some(p) = param {
            if let Some(name) = &p.name {
                let original = name.name.clone();
                let normalized = normalize_param_name(&original);
                if original != normalized {
                    map.insert(original, normalized);
                }
            }
        }
    }
    map
}

/// Check if a member access expression is a simple name path (only Variables and MemberAccesses)
/// that can be flattened to a single string like "ResourceMetering.ResourceConfig"
fn is_simple_member_path(expr: &Expression) -> bool {
    match expr {
        Expression::MemberAccess(_, e, _) => is_simple_member_path(e),
        Expression::Variable(_) => true,
        Expression::Type(_, _) => true,
        _ => false,
    }
}

/// Collect a member access chain into a single string (e.g., "ResourceMetering.ResourceConfig")
/// This ensures the full path is treated as an indivisible unit that won't break mid-name.
/// Only call this after verifying with is_simple_member_path().
fn collect_member_access_path(expr: &Expression, renames: &HashMap<String, String>) -> String {
    match expr {
        Expression::MemberAccess(_, e, member) => {
            format!("{}.{}", collect_member_access_path(e, renames), member.name)
        }
        Expression::Variable(ident) => {
            // Apply rename if present
            renames.get(&ident.name).cloned().unwrap_or_else(|| ident.name.clone())
        }
        Expression::Type(_, ty) => format_type_to_string(ty),
        _ => unreachable!("collect_member_access_path called on non-simple path"),
    }
}

/// Check if an expression is "complex" enough to warrant hard line breaks.
/// Complex expressions include array subscripts, member access chains with subscripts, etc.
/// These tend to create long lines that benefit from consistent multi-line formatting.
fn expression_is_complex(expr: &Expression) -> bool {
    match expr {
        Expression::ArraySubscript(_, _, _) => true,
        Expression::MemberAccess(_, base, _) => expression_is_complex(base),
        Expression::FunctionCall(_, callee, _) => expression_is_complex(callee),
        _ => false,
    }
}

/// Format a type to a string (for use in member access paths)
fn format_type_to_string(ty: &Type) -> String {
    match ty {
        Type::Address => "address".to_string(),
        Type::AddressPayable => "address payable".to_string(),
        Type::Bool => "bool".to_string(),
        Type::String => "string".to_string(),
        Type::Bytes(n) => format!("bytes{}", n),
        Type::Int(n) => format!("int{}", n),
        Type::Uint(n) => format!("uint{}", n),
        Type::DynamicBytes => "bytes".to_string(),
        Type::Mapping {
            key,
            key_name,
            value,
            value_name,
            ..
        } => {
            let key_str = format_expression_to_string(key);
            let key_name_str = key_name.as_ref().map(|n| {
                // Don't prefix "TODO" placeholder with underscore
                if n.name.starts_with('_') || n.name == "TODO" {
                    n.name.clone()
                } else {
                    format!("_{}", n.name)
                }
            });
            let value_str = format_expression_to_string(value);
            let value_name_str = value_name.as_ref().map(|n| {
                // Don't prefix "TODO" placeholder with underscore
                if n.name.starts_with('_') || n.name == "TODO" {
                    n.name.clone()
                } else {
                    format!("_{}", n.name)
                }
            });

            match (key_name_str, value_name_str) {
                (Some(kn), Some(vn)) => {
                    format!("mapping({} {} => {} {})", key_str, kn, value_str, vn)
                }
                (Some(kn), None) => format!("mapping({} {} => {})", key_str, kn, value_str),
                (None, Some(vn)) => format!("mapping({} => {} {})", key_str, value_str, vn),
                (None, None) => format!("mapping({} => {})", key_str, value_str),
            }
        }
        Type::Function { .. } => "function".to_string(),
        Type::Rational => "rational".to_string(),
        Type::Payable => "payable".to_string(),
    }
}

/// Format an expression to a string (for use in type contexts)
fn format_expression_to_string(expr: &Expression) -> String {
    match expr {
        Expression::Variable(ident) => ident.name.clone(),
        Expression::MemberAccess(_, _, _) => collect_member_access_path(expr, &HashMap::new()),
        Expression::Type(_, ty) => format_type_to_string(ty),
        _ => "?".to_string(),
    }
}

/// Format any expression into IR elements
pub fn format_expression(expr: &Expression) -> IRElement {
    format_expression_with_renames(expr, &HashMap::new())
}

/// Format any expression into IR elements, applying variable renames
pub fn format_expression_with_renames(
    expr: &Expression,
    renames: &HashMap<String, String>,
) -> IRElement {
    // Helper closure to recurse with same renames
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);

    match expr {
        Expression::Type(_, ty) => format_type(ty),
        Expression::Variable(ident) => {
            // Apply rename if present
            if let Some(new_name) = renames.get(&ident.name) {
                IRElement::text(new_name)
            } else {
                IRElement::text(&ident.name)
            }
        }
        Expression::BoolLiteral(_, val) => IRElement::text(if *val { "true" } else { "false" }),
        Expression::NumberLiteral(loc, num, exp, unit) => format_number_literal(loc, num, exp, unit),
        Expression::HexNumberLiteral(_, hex, _) => IRElement::text(hex),
        Expression::StringLiteral(strings) => format_string_literal(strings),
        Expression::HexLiteral(hexes) => format_hex_literal(hexes),
        Expression::AddressLiteral(_, addr) => IRElement::text(addr),
        Expression::ArrayLiteral(_, elements) => {
            if elements.is_empty() {
                IRElement::text("[]")
            } else {
                let mut ir = vec![IRElement::text("[")];
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        ir.push(IRElement::text(","));
                        ir.push(IRElement::SoftLineBreak);
                    }
                    ir.push(fmt(elem));
                }
                ir.push(IRElement::text("]"));
                IRElement::group(ir)
            }
        }
        Expression::List(_, params) => {
            let mut ir = vec![IRElement::text("(")];
            for (i, (_, param)) in params.iter().enumerate() {
                if i > 0 {
                    ir.push(IRElement::text(","));
                    ir.push(IRElement::SoftLineBreak);
                }
                if let Some(p) = param {
                    // Format parameter with renames applied to its type expression
                    ir.push(format_expression_with_renames(&p.ty, renames));
                    if let Some(storage) = &p.storage {
                        ir.push(IRElement::text(" "));
                        ir.push(format_storage_location(storage));
                    }
                    if let Some(name) = &p.name {
                        ir.push(IRElement::text(" "));
                        ir.push(IRElement::text(&normalize_param_name(&name.name)));
                    }
                }
            }
            ir.push(IRElement::text(")"));
            IRElement::group(ir)
        }
        Expression::ArraySubscript(_, array, index) => {
            format_array_subscript(array, index.as_ref().map(|e| e.as_ref()), renames)
        }
        Expression::MemberAccess(_, e, member) => {
            // For simple name paths (like ResourceMetering.ResourceConfig), collect into
            // a single indivisible string so it won't break mid-name
            if is_simple_member_path(expr) {
                let path = collect_member_access_path(expr, renames);
                IRElement::text(path)
            } else {
                // For complex expressions (like _slots[i].key), keep as separate elements
                IRElement::group(vec![
                    fmt(e),
                    IRElement::text("."),
                    IRElement::text(&member.name),
                ])
            }
        }
        Expression::FunctionCall(_, func, args) => {
            let mut ir = vec![fmt(func), IRElement::text("(")];
            if !args.is_empty() {
                // Build args directly into indent content (no separate Group)
                // so they inherit break mode when outer call breaks
                let mut indent_content = vec![IRElement::SoftestLineBreak];
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        indent_content.push(IRElement::text(","));
                        indent_content.push(IRElement::SoftLineBreak);
                    }
                    indent_content.push(fmt(arg));
                }
                ir.push(IRElement::indent(indent_content));
                ir.push(IRElement::SoftestLineBreak);
            }
            ir.push(IRElement::text(")"));
            IRElement::group(ir)
        }
        Expression::NamedFunctionCall(_, func, args) => {
            // Each field on its own line, indented
            let mut ir = vec![fmt(func), IRElement::text("({")];
            if !args.is_empty() {
                for (i, arg) in args.iter().enumerate() {
                    ir.push(IRElement::HardLineBreak);
                    ir.push(IRElement::indent(vec![
                        IRElement::text(&arg.name.name),
                        IRElement::text(": "),
                        fmt(&arg.expr),
                        if i < args.len() - 1 {
                            IRElement::text(",")
                        } else {
                            IRElement::text("")
                        },
                    ]));
                }
                ir.push(IRElement::HardLineBreak);
            }
            ir.push(IRElement::text("})"));
            IRElement::group(ir)
        }
        Expression::FunctionCallBlock(_, func, stmt) => {
            // Format as func{ args }() - the stmt is usually Statement::Args
            // Use HardLineBreak for complex args (subscripts etc), SoftLineBreak for simple
            let mut ir = vec![fmt(func), IRElement::text("{")];
            match stmt.as_ref() {
                Statement::Args(_, args) => {
                    if !args.is_empty() {
                        // Check if any arg has complex expressions (subscripts, member access chains)
                        // These tend to create long lines that need consistent breaking
                        let needs_hard_breaks = args.iter().any(|arg| expression_is_complex(&arg.expr));

                        let mut args_ir = vec![];
                        for (i, arg) in args.iter().enumerate() {
                            if i > 0 {
                                args_ir.push(IRElement::text(","));
                            }
                            if needs_hard_breaks {
                                args_ir.push(IRElement::HardLineBreak);
                            } else {
                                args_ir.push(IRElement::SoftLineBreak);
                            }
                            args_ir.push(IRElement::text(&arg.name.name));
                            args_ir.push(IRElement::text(": "));
                            args_ir.push(fmt(&arg.expr));
                        }
                        ir.push(IRElement::indent(args_ir));
                        if needs_hard_breaks {
                            ir.push(IRElement::HardLineBreak);
                        } else {
                            ir.push(IRElement::SoftLineBreak);
                        }
                    }
                    ir.push(IRElement::text("}"));
                }
                _ => {
                    // Fallback for other statement types
                    ir.push(IRElement::text(" /* block */ }"));
                }
            }
            IRElement::group(ir)
        }
        Expression::PreIncrement(_, e) => IRElement::group(vec![IRElement::text("++"), fmt(e)]),
        Expression::PostIncrement(_, e) => IRElement::group(vec![fmt(e), IRElement::text("++")]),
        Expression::PreDecrement(_, e) => IRElement::group(vec![IRElement::text("--"), fmt(e)]),
        Expression::PostDecrement(_, e) => IRElement::group(vec![fmt(e), IRElement::text("--")]),
        Expression::Not(_, e) => IRElement::group(vec![IRElement::text("!"), fmt(e)]),
        Expression::BitwiseNot(_, e) => IRElement::group(vec![IRElement::text("~"), fmt(e)]),
        Expression::UnaryPlus(_, e) => IRElement::group(vec![IRElement::text("+"), fmt(e)]),
        Expression::Negate(_, e) => IRElement::group(vec![IRElement::text("-"), fmt(e)]),
        Expression::Power(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" ** "), fmt(r)])
        }
        Expression::Multiply(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" * "), fmt(r)])
        }
        Expression::Divide(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" / "), fmt(r)])
        }
        Expression::Modulo(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" % "), fmt(r)])
        }
        Expression::Add(_, l, r) => IRElement::group(vec![fmt(l), IRElement::text(" + "), fmt(r)]),
        Expression::Subtract(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" - "), fmt(r)])
        }
        Expression::ShiftLeft(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" << "), fmt(r)])
        }
        Expression::ShiftRight(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" >> "), fmt(r)])
        }
        Expression::Less(_, l, r) => IRElement::group(vec![fmt(l), IRElement::text(" < "), fmt(r)]),
        Expression::More(_, l, r) => IRElement::group(vec![fmt(l), IRElement::text(" > "), fmt(r)]),
        Expression::LessEqual(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" <= "), fmt(r)])
        }
        Expression::MoreEqual(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" >= "), fmt(r)])
        }
        Expression::Equal(_, l, r) => IRElement::group(vec![
            fmt(l),
            IRElement::text(" == "),
            IRElement::SoftestLineBreak,
            fmt(r),
        ]),
        Expression::NotEqual(_, l, r) => IRElement::group(vec![
            fmt(l),
            IRElement::text(" != "),
            IRElement::SoftestLineBreak,
            fmt(r),
        ]),
        Expression::BitwiseAnd(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" & "), fmt(r)])
        }
        Expression::BitwiseXor(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" ^ "), fmt(r)])
        }
        Expression::BitwiseOr(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" | "), fmt(r)])
        }
        Expression::And(_, l, r) => IRElement::group(vec![
            fmt(l),
            IRElement::SoftLineBreak,
            IRElement::text("&& "),
            fmt(r),
        ]),
        Expression::Or(_, l, r) => IRElement::group(vec![
            fmt(l),
            IRElement::SoftLineBreak,
            IRElement::text("|| "),
            fmt(r),
        ]),
        Expression::ConditionalOperator(_, cond, true_expr, false_expr) => IRElement::group(vec![
            fmt(cond),
            IRElement::text(" ? "),
            fmt(true_expr),
            IRElement::text(" : "),
            fmt(false_expr),
        ]),
        Expression::Assign(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" = "), fmt(r)])
        }
        Expression::AssignOr(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" |= "), fmt(r)])
        }
        Expression::AssignAnd(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" &= "), fmt(r)])
        }
        Expression::AssignXor(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" ^= "), fmt(r)])
        }
        Expression::AssignShiftLeft(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" <<= "), fmt(r)])
        }
        Expression::AssignShiftRight(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" >>= "), fmt(r)])
        }
        Expression::AssignAdd(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" += "), fmt(r)])
        }
        Expression::AssignSubtract(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" -= "), fmt(r)])
        }
        Expression::AssignMultiply(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" *= "), fmt(r)])
        }
        Expression::AssignDivide(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" /= "), fmt(r)])
        }
        Expression::AssignModulo(_, l, r) => {
            IRElement::group(vec![fmt(l), IRElement::text(" %= "), fmt(r)])
        }
        Expression::New(_, e) => IRElement::group(vec![IRElement::text("new "), fmt(e)]),
        Expression::Delete(_, e) => IRElement::group(vec![IRElement::text("delete "), fmt(e)]),
        Expression::Parenthesis(_, e) => IRElement::group(vec![
            IRElement::text("("),
            IRElement::indent(vec![IRElement::SoftestLineBreak, fmt(e)]),
            IRElement::SoftestLineBreak,
            IRElement::text(")"),
        ]),
        Expression::ArraySlice(_, array, start, end) => {
            let mut ir = vec![fmt(array), IRElement::text("[")];
            if let Some(s) = start {
                ir.push(fmt(s));
            }
            ir.push(IRElement::text(":"));
            if let Some(e) = end {
                ir.push(fmt(e));
            }
            ir.push(IRElement::text("]"));
            IRElement::group(ir)
        }
        Expression::RationalNumberLiteral(_, mantissa, exponent, exp_str, _) => {
            format_rational_number_literal(mantissa, exponent, exp_str)
        }
    }
}

/// Format a type expression
pub fn format_type(ty: &Type) -> IRElement {
    match ty {
        Type::Address => IRElement::text("address"),
        Type::AddressPayable => IRElement::text("address payable"),
        Type::Payable => IRElement::text("payable"),
        Type::Bool => IRElement::text("bool"),
        Type::String => IRElement::text("string"),
        Type::Int(n) => IRElement::text(&format!("int{}", n)),
        Type::Uint(n) => IRElement::text(&format!("uint{}", n)),
        Type::Bytes(n) => IRElement::text(&format!("bytes{}", n)),
        Type::Rational => IRElement::text("rational"),
        Type::DynamicBytes => IRElement::text("bytes"),
        Type::Mapping {
            key,
            key_name,
            value,
            value_name,
            ..
        } => {
            // Format mapping with multi-line layout and underscore-prefixed names
            let mut ir = vec![IRElement::text("mapping (")];

            let mut params_ir = vec![IRElement::HardLineBreak];

            // Key type and name (use TODO if no name)
            params_ir.push(format_expression(key));
            params_ir.push(IRElement::text(" "));
            if let Some(name) = key_name {
                // Don't prefix "TODO" placeholder with underscore
                let prefixed = if name.name.starts_with('_') || name.name == "TODO" {
                    name.name.clone()
                } else {
                    format!("_{}", name.name)
                };
                params_ir.push(IRElement::text(&prefixed));
            } else {
                params_ir.push(IRElement::text("TODO"));
            }

            params_ir.push(IRElement::text(" => "));

            // Value type and name (use TODO if no name, unless it's a nested mapping)
            params_ir.push(format_expression(value));
            let is_nested_mapping =
                matches!(value.as_ref(), Expression::Type(_, Type::Mapping { .. }));
            if !is_nested_mapping {
                params_ir.push(IRElement::text(" "));
                if let Some(name) = value_name {
                    // Don't prefix "TODO" placeholder with underscore
                    let prefixed = if name.name.starts_with('_') || name.name == "TODO" {
                        name.name.clone()
                    } else {
                        format!("_{}", name.name)
                    };
                    params_ir.push(IRElement::text(&prefixed));
                } else {
                    params_ir.push(IRElement::text("TODO"));
                }
            }

            ir.push(IRElement::indent(params_ir));
            ir.push(IRElement::HardLineBreak);
            ir.push(IRElement::text(")"));

            IRElement::group(ir)
        }
        Type::Function {
            params,
            attributes,
            returns,
            ..
        } => {
            let returns_params = returns
                .as_ref()
                .map(|(params, _)| params.as_slice())
                .unwrap_or(&[]);
            format_function_type(params, attributes, returns_params)
        }
    }
}

fn format_number_literal(loc: &Loc, num: &str, exp: &str, unit: &Option<Identifier>) -> IRElement {
    // Try to extract the original text from source to preserve underscores
    if let Some(original) = extract_source_text(loc) {
        // The original includes the full number literal as written in source
        return IRElement::text(original);
    }

    // Fallback to reconstructed text (without underscores)
    let mut text = num.to_string();
    if !exp.is_empty() {
        text.push_str(exp);
    }
    if let Some(u) = unit {
        text.push(' ');
        text.push_str(&u.name);
    }
    IRElement::text(text)
}

fn format_string_literal(strings: &[StringLiteral]) -> IRElement {
    let parts: Vec<String> = strings
        .iter()
        .map(|s| format!("\"{}\"", s.string))
        .collect();
    IRElement::text(parts.join(" "))
}

fn format_hex_literal(hexes: &[HexLiteral]) -> IRElement {
    let parts: Vec<String> = hexes.iter().map(|h| format!("hex\"{}\"", h.hex)).collect();
    IRElement::text(parts.join(" "))
}

pub fn format_parameter(param: &Parameter) -> Vec<IRElement> {
    format_parameter_with_options(param, true, false)
}

/// Format a parameter with full options
/// - normalize: add underscore prefix to names without one
/// - strip_name: omit the parameter name entirely (for return types)
pub fn format_parameter_with_options(
    param: &Parameter,
    normalize: bool,
    strip_name: bool,
) -> Vec<IRElement> {
    let mut ir = vec![];

    // Type
    ir.push(format_expression(&param.ty));

    // Storage location
    if let Some(storage) = &param.storage {
        ir.push(IRElement::text(" "));
        ir.push(format_storage_location(storage));
    }

    // Name (normalized if requested, omitted if strip_name)
    if !strip_name {
        if let Some(name) = &param.name {
            ir.push(IRElement::text(" "));
            if normalize {
                ir.push(IRElement::text(&normalize_param_name(&name.name)));
            } else {
                ir.push(IRElement::text(&name.name));
            }
        }
    }

    ir
}

pub fn format_storage_location(storage: &StorageLocation) -> IRElement {
    IRElement::text(match storage {
        StorageLocation::Memory(_) => "memory",
        StorageLocation::Storage(_) => "storage",
        StorageLocation::Calldata(_) => "calldata",
    })
}

fn format_array_subscript(array: &Expression, index: Option<&Expression>, renames: &HashMap<String, String>) -> IRElement {
    // For simple type arrays like bytes32[], format as single text element
    if index.is_none() {
        match array {
            Expression::Variable(var) => {
                // Simple type array - format as single text element
                let name = renames.get(&var.name).cloned().unwrap_or_else(|| var.name.clone());
                return IRElement::text(&format!("{}[]", name));
            }
            Expression::Type(_, ty) => {
                // Type array - format the type followed by []
                let type_str = match ty {
                    Type::Uint(n) => format!("uint{}[]", n),
                    Type::Int(n) => format!("int{}[]", n),
                    Type::Bytes(n) => format!("bytes{}[]", n),
                    Type::Address => "address[]".to_string(),
                    Type::Bool => "bool[]".to_string(),
                    Type::String => "string[]".to_string(),
                    Type::DynamicBytes => "bytes[]".to_string(),
                    _ => {
                        // For complex types, fall back to default formatting
                        let ir = vec![
                            format_expression_with_renames(array, renames),
                            IRElement::text("["),
                            IRElement::text("]"),
                        ];
                        return IRElement::group(ir);
                    }
                };
                return IRElement::text(&type_str);
            }
            _ => {}
        }
    }

    // For mapping/array access, try to collect entire chain as single text element
    // to prevent breaking in the middle of access chains like challenges[x][y]
    if let Some(text) = try_collect_subscript_chain(array, index, renames) {
        return IRElement::text(text);
    }

    // Fall back to grouped format for complex expressions
    let mut ir = vec![format_expression_with_renames(array, renames), IRElement::text("[")];

    if let Some(idx) = index {
        ir.push(format_expression_with_renames(idx, renames));
    }

    ir.push(IRElement::text("]"));
    IRElement::group(ir)
}

/// Try to collect an array subscript chain into a single text string.
/// Returns None if the expression is too complex to represent as simple text.
fn try_collect_subscript_chain(array: &Expression, index: Option<&Expression>, renames: &HashMap<String, String>) -> Option<String> {
    let mut result = String::new();

    // Collect the base array expression
    match array {
        Expression::Variable(var) => {
            let name = renames.get(&var.name).cloned().unwrap_or_else(|| var.name.clone());
            result.push_str(&name);
        }
        Expression::ArraySubscript(_, inner_array, inner_index) => {
            // Recursively collect inner subscript
            let inner =
                try_collect_subscript_chain(inner_array, inner_index.as_ref().map(|e| e.as_ref()), renames)?;
            result.push_str(&inner);
        }
        Expression::MemberAccess(_, base, member) => {
            // Handle member access like _challenge.challenger
            let base_text = try_collect_simple_expr(base, renames)?;
            result.push_str(&base_text);
            result.push('.');
            result.push_str(&member.name);
        }
        _ => return None, // Complex expression, can't simplify
    }

    // Add the index
    result.push('[');
    if let Some(idx) = index {
        let idx_text = try_collect_simple_expr(idx, renames)?;
        result.push_str(&idx_text);
    }
    result.push(']');

    Some(result)
}

/// Try to collect a simple expression into text.
fn try_collect_simple_expr(expr: &Expression, renames: &HashMap<String, String>) -> Option<String> {
    match expr {
        Expression::Variable(var) => {
            Some(renames.get(&var.name).cloned().unwrap_or_else(|| var.name.clone()))
        }
        Expression::MemberAccess(_, base, member) => {
            let base_text = try_collect_simple_expr(base, renames)?;
            Some(format!("{}.{}", base_text, member.name))
        }
        Expression::ArraySubscript(_, array, index) => {
            try_collect_subscript_chain(array, index.as_ref().map(|e| e.as_ref()), renames)
        }
        _ => None,
    }
}

fn format_function_type(
    params: &[(Loc, Option<Parameter>)],
    attributes: &[FunctionAttribute],
    returns: &[(Loc, Option<Parameter>)],
) -> IRElement {
    let mut ir = vec![IRElement::text("function(")];

    // Parameters
    for (i, (_, param)) in params.iter().enumerate() {
        if i > 0 {
            ir.push(IRElement::text(", "));
        }
        if let Some(p) = param {
            ir.extend(format_parameter(p));
        }
    }
    ir.push(IRElement::text(")"));

    // Attributes
    for attr in attributes {
        ir.push(IRElement::text(" "));
        ir.push(format_function_attribute(attr));
    }

    // Returns
    if !returns.is_empty() {
        ir.push(IRElement::text(" returns ("));
        for (i, (_, param)) in returns.iter().enumerate() {
            if i > 0 {
                ir.push(IRElement::text(", "));
            }
            if let Some(p) = param {
                ir.extend(format_parameter(p));
            }
        }
        ir.push(IRElement::text(")"));
    }

    IRElement::group(ir)
}

fn format_function_attribute(attr: &FunctionAttribute) -> IRElement {
    match attr {
        FunctionAttribute::Visibility(vis) => format_visibility(vis),
        FunctionAttribute::Mutability(mut_) => format_mutability(mut_),
        FunctionAttribute::Virtual(_) => IRElement::text("virtual"),
        FunctionAttribute::Immutable(_) => IRElement::text("immutable"),
        FunctionAttribute::Override(_, _) => IRElement::text("override"),
        FunctionAttribute::BaseOrModifier(_, base) => format_base_or_modifier(base),
        FunctionAttribute::Error(_) => IRElement::text("/* error */"),
    }
}

pub fn format_visibility(vis: &Visibility) -> IRElement {
    IRElement::text(match vis {
        Visibility::Public(_) => "public",
        Visibility::Private(_) => "private",
        Visibility::Internal(_) => "internal",
        Visibility::External(_) => "external",
    })
}

fn format_mutability(mut_: &Mutability) -> IRElement {
    IRElement::text(match mut_ {
        Mutability::Pure(_) => "pure",
        Mutability::View(_) => "view",
        Mutability::Constant(_) => "constant",
        Mutability::Payable(_) => "payable",
    })
}

fn format_base_or_modifier(base: &Base) -> IRElement {
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

pub fn format_identifier_path(path: &IdentifierPath) -> IRElement {
    let parts: Vec<String> = path.identifiers.iter().map(|id| id.name.clone()).collect();
    IRElement::text(parts.join("."))
}

fn format_rational_number_literal(mantissa: &str, exponent: &str, exp_str: &str) -> IRElement {
    let mut text = mantissa.to_string();
    if !exponent.is_empty() || !exp_str.is_empty() {
        text.push_str(exp_str);
    }
    IRElement::text(text)
}
