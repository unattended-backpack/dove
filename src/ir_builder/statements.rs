//! Statement formatting for the IR builder

use super::ir::IRElement;
use crate::ir_builder::comments::*;
use crate::ir_builder::expressions::*;
use crate::collector::get_loc_start;
use crate::collector::model::*;
use solang_parser::helpers::CodeLocation;
use solang_parser::pt::*;
use std::collections::{HashMap, HashSet};

/// Context for return variable handling
pub struct ReturnVarContext<'a> {
    /// Maps return var name to its Parameter (for type info)
    pub return_var_types: &'a HashMap<String, Parameter>,
    /// Tracks which return vars have been declared (first assignment converts to declaration)
    pub declared_return_vars: &'a mut HashSet<String>,
    /// Return vars that are only assigned once - these use direct `return expr;`
    pub single_assignment_vars: &'a HashSet<String>,
}

/// Build IR for a statement with variable renames and return variable conversion
pub fn build_statement_ir_full(
    stmt: &CommentedStatement,
    renames: &mut HashMap<String, String>,
    return_ctx: &mut ReturnVarContext,
) -> Vec<IRElement> {
    with_comments(&stmt.leading_comments, &stmt.trailing_comments, || {
        match &stmt.statement {
            Statement::Assembly {
                loc,
                dialect,
                flags,
                block,
            } => {
                // Use the collected YUL block if available for proper comment handling
                if let Some(collected_yul) = &stmt.yul_block {
                    format_assembly_with_collected_and_renames(
                        loc,
                        dialect.as_ref(),
                        flags.as_ref(),
                        collected_yul,
                        renames,
                    )
                } else {
                    format_assembly(loc, dialect.as_ref(), flags.as_ref(), block)
                }
            }
            Statement::If(_, cond, then_stmt, else_stmt) => {
                // Build if statement with collected nested statements
                build_if_ir_full(
                    cond,
                    then_stmt,
                    else_stmt.as_deref(),
                    &stmt.nested_statements,
                    renames,
                    return_ctx,
                )
            }
            Statement::Block { .. } => {
                // Handle block with collected statements
                if let Some(nested) = &stmt.nested_statements {
                    format_block_full(nested, renames, return_ctx)
                } else {
                    format_statement_full(&stmt.statement, renames, return_ctx)
                }
            }
            _ => format_statement_full(&stmt.statement, renames, return_ctx),
        }
    })
}

pub fn format_statement_with_renames(
    stmt: &Statement,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let empty_types = HashMap::new();
    let mut empty_declared = HashSet::new();
    let empty_single = HashSet::new();
    let mut ctx = ReturnVarContext {
        return_var_types: &empty_types,
        declared_return_vars: &mut empty_declared,
        single_assignment_vars: &empty_single,
    };
    format_statement_full(stmt, renames, &mut ctx)
}

/// Format a statement with renames and return variable conversion
/// First assignment to a return var becomes a declaration; subsequent assignments stay as-is
pub fn format_statement_full(
    stmt: &Statement,
    renames: &mut HashMap<String, String>,
    return_ctx: &mut ReturnVarContext,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);

    match stmt {
        Statement::Block { statements, .. } => {
            // Convert Vec<Statement> to block format
            let mut ir = vec![IRElement::text("{")];

            if !statements.is_empty() {
                for stmt in statements {
                    ir.push(IRElement::HardLineBreak);
                    let stmt_ir = format_statement_full(stmt, renames, return_ctx);
                    ir.push(IRElement::indent(stmt_ir));
                }
                ir.push(IRElement::HardLineBreak);
            }

            ir.push(IRElement::text("}"));
            ir
        }
        Statement::Assembly {
            loc,
            dialect,
            flags,
            block,
        } => format_assembly(loc, dialect.as_ref(), flags.as_ref(), block),
        Statement::Args(_, args) => format_args_with_renames(args, renames),
        Statement::If(_, cond, then_stmt, else_stmt) => {
            format_if_with_renames(cond, then_stmt, else_stmt.as_deref(), renames)
        }
        Statement::While(_, cond, body) => format_while_with_renames(cond, body, renames),
        Statement::Expression(_, expr) => {
            // Check if this is a tuple destructuring assignment like (bool success, bytes memory result) = ...
            // We need to extract variable names and add them to renames BEFORE formatting
            if let Expression::Assign(_, left, _) = expr {
                if let Expression::List(_, params) = left.as_ref() {
                    // Tuple destructuring - extract variable names and add to renames
                    for (_, param) in params {
                        if let Some(p) = param {
                            if let Some(name) = &p.name {
                                let original = name.name.clone();
                                if original.len() > 1 {
                                    let normalized = normalize_param_name(&original);
                                    if original != normalized {
                                        renames.insert(original, normalized);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            // Check if this is an assignment to a return variable
            if let Expression::Assign(_, left, right) = expr {
                if let Expression::Variable(var) = left.as_ref() {
                    if return_ctx.return_var_types.contains_key(&var.name) {
                        // Check if this is a single-assignment var - use direct return
                        if return_ctx.single_assignment_vars.contains(&var.name) {
                            return vec![
                                IRElement::text("return"),
                                IRElement::text(" "),
                                format_expression_with_renames(right, renames),
                                IRElement::text(";"),
                            ];
                        }

                        // Multi-assignment var - first becomes declaration, rest stay as assignments
                        if let Some(param) = return_ctx.return_var_types.get(&var.name) {
                            if !return_ctx.declared_return_vars.contains(&var.name) {
                                // First assignment - convert to variable declaration
                                return_ctx.declared_return_vars.insert(var.name.clone());
                                let mut ir = vec![];
                                // Format the type
                                ir.push(format_expression(&param.ty));
                                // Add storage location if present
                                if let Some(storage) = &param.storage {
                                    ir.push(IRElement::text(" "));
                                    ir.push(format_storage_location(storage));
                                }
                                // Normalize the variable name (add underscore prefix)
                                let original = var.name.clone();
                                let normalized = normalize_param_name(&original);
                                if original != normalized {
                                    renames.insert(original, normalized.clone());
                                }
                                // Add variable name and initializer
                                ir.push(IRElement::text(" "));
                                ir.push(IRElement::text(&normalized));
                                ir.push(IRElement::text(" = "));
                                // Call directly to avoid borrow conflict with the closure
                                ir.push(format_expression_with_renames(right, renames));
                                ir.push(IRElement::text(";"));
                                return ir;
                            }
                        }
                        // Subsequent assignment - keep as regular assignment
                    }
                }
            }
            let mut ir = vec![format_expression_with_renames(expr, renames)];
            ir.push(IRElement::text(";"));
            ir
        }
        Statement::VariableDefinition(_, decl, init) => {
            format_variable_definition_with_renames(decl, init.as_ref(), renames)
        }
        Statement::For(_, init, cond, update, body) => format_for_with_renames(
            init.as_ref().map(|s| &**s),
            cond.as_ref().map(|e| &**e),
            update.as_ref().map(|e| &**e),
            body.as_ref().map(|s| &**s),
            renames,
        ),
        Statement::DoWhile(_, body, cond) => format_do_while_with_renames(body, cond, renames),
        Statement::Continue(_) => vec![IRElement::text("continue;")],
        Statement::Break(_) => vec![IRElement::text("break;")],
        Statement::Return(_, val) => format_return_with_renames(val.as_ref(), renames),
        Statement::Revert(_, error, args) => {
            format_revert_with_renames(error.as_ref(), args, renames)
        }
        Statement::RevertNamedArgs(_, error, args) => {
            format_revert_named_with_renames(error.as_ref(), args, renames)
        }
        Statement::Emit(_, event) => {
            vec![
                IRElement::text("emit"),
                IRElement::text(" "),
                fmt(event),
                IRElement::text(";"),
            ]
        }
        Statement::Try(_, expr, returns_and_body, clauses) => {
            let returns = returns_and_body
                .as_ref()
                .map(|(returns, _)| returns.as_slice())
                .unwrap_or(&[]);
            format_try_with_renames(expr, returns, clauses, renames)
        }
        Statement::Error(_) => vec![IRElement::text("/* error statement */")],
    }
}

fn format_block_full(
    statements: &[CommentedStatement],
    renames: &mut HashMap<String, String>,
    return_ctx: &mut ReturnVarContext,
) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("{")];

    if !statements.is_empty() {
        for stmt in statements {
            ir.push(IRElement::HardLineBreak);
            let stmt_ir = build_statement_ir_full(stmt, renames, return_ctx);
            ir.push(IRElement::indent(stmt_ir));
        }
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text("}"));
    ir
}

fn format_args_with_renames(
    args: &[NamedArgument],
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);
    let mut ir = vec![IRElement::text("{")];

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            ir.push(IRElement::text(","));
            ir.push(IRElement::SoftLineBreak);
        }
        ir.push(IRElement::text(&arg.name.name));
        ir.push(IRElement::text(": "));
        ir.push(fmt(&arg.expr));
    }

    ir.push(IRElement::text("}"));
    vec![IRElement::group(ir)]
}

fn format_if_with_renames(
    cond: &Expression,
    then_stmt: &Statement,
    else_stmt: Option<&Statement>,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);

    // Build the if header with SoftestLineBreaks around condition
    let header = vec![
        IRElement::text("if ("),
        IRElement::indent(vec![IRElement::SoftestLineBreak, fmt(cond)]),
        IRElement::SoftestLineBreak,
        IRElement::text(") "),
    ];

    let mut ir = vec![IRElement::group(header)];
    ir.extend(format_statement_with_renames(then_stmt, renames));

    if let Some(else_s) = else_stmt {
        ir.push(IRElement::text(" else "));
        ir.extend(format_statement_with_renames(else_s, renames));
    }

    ir
}

fn build_if_ir_full(
    cond: &Expression,
    then_stmt: &Statement,
    else_stmt: Option<&Statement>,
    nested_statements: &Option<Vec<CommentedStatement>>,
    renames: &mut HashMap<String, String>,
    return_ctx: &mut ReturnVarContext,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);

    // Build the if header with SoftestLineBreaks around condition
    let header = vec![
        IRElement::text("if ("),
        IRElement::indent(vec![IRElement::SoftestLineBreak, fmt(cond)]),
        IRElement::SoftestLineBreak,
        IRElement::text(") "),
    ];

    let mut ir = vec![IRElement::group(header)];

    // Handle the then branch
    match then_stmt {
        Statement::Block { .. } => {
            // If we have collected nested statements, use them
            if let Some(nested) = nested_statements {
                // Format as a block with the collected statements
                let mut block_ir = vec![IRElement::text("{")];

                if !nested.is_empty() {
                    for (_i, stmt) in nested.iter().enumerate() {
                        block_ir.push(IRElement::HardLineBreak);

                        // Add blank line before statements with leading comments
                        if !stmt.leading_comments.is_empty() {
                            block_ir.push(IRElement::HardLineBreak);
                        }

                        let stmt_ir = build_statement_ir_full(stmt, renames, return_ctx);
                        block_ir.push(IRElement::indent(stmt_ir));
                    }
                    block_ir.push(IRElement::HardLineBreak);
                }

                block_ir.push(IRElement::text("}"));
                ir.extend(block_ir);
            } else {
                // Fall back to regular formatting
                ir.extend(format_statement_full(then_stmt, renames, return_ctx));
            }
        }
        _ => {
            // Non-block statement - wrap in braces for consistent style
            let mut block_ir = vec![IRElement::text("{")];
            block_ir.push(IRElement::HardLineBreak);
            let stmt_ir = format_statement_full(then_stmt, renames, return_ctx);
            block_ir.push(IRElement::indent(stmt_ir));
            block_ir.push(IRElement::HardLineBreak);
            block_ir.push(IRElement::text("}"));
            ir.extend(block_ir);
        }
    }

    if let Some(else_s) = else_stmt {
        ir.push(IRElement::text(" else "));
        // Wrap else body in braces too (unless it's already a block or if-else chain)
        match else_s {
            Statement::Block { .. } => {
                ir.extend(format_statement_full(else_s, renames, return_ctx));
            }
            Statement::If(_, _, _, _) => {
                // if-else chain - don't wrap
                ir.extend(format_statement_full(else_s, renames, return_ctx));
            }
            _ => {
                // Non-block else - wrap in braces
                let mut block_ir = vec![IRElement::text("{")];
                block_ir.push(IRElement::HardLineBreak);
                let stmt_ir = format_statement_full(else_s, renames, return_ctx);
                block_ir.push(IRElement::indent(stmt_ir));
                block_ir.push(IRElement::HardLineBreak);
                block_ir.push(IRElement::text("}"));
                ir.extend(block_ir);
            }
        }
    }

    ir
}

fn format_while_with_renames(
    cond: &Expression,
    body: &Statement,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);
    let mut ir = vec![IRElement::text("while ("), fmt(cond), IRElement::text(") ")];

    ir.extend(format_statement_with_renames(body, renames));
    ir
}

fn format_variable_definition_with_renames(
    decl: &VariableDeclaration,
    init: Option<&Expression>,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let mut ir = vec![];

    // Check if this is a tuple destructuring (type is Expression::List)
    if let Expression::List(_, params) = &decl.ty {
        // Tuple destructuring: (bool success, bytes memory result) = ...
        ir.push(IRElement::text("("));
        for (i, (_, param)) in params.iter().enumerate() {
            if i > 0 {
                ir.push(IRElement::text(", "));
            }
            if let Some(p) = param {
                // Format type
                ir.push(format_expression(&p.ty));
                // Storage location
                if let Some(storage) = &p.storage {
                    ir.push(IRElement::text(" "));
                    ir.push(format_storage_location(storage));
                }
                // Name with normalization and renames tracking
                if let Some(name) = &p.name {
                    ir.push(IRElement::text(" "));
                    let original = name.name.clone();
                    if original.len() == 1 {
                        ir.push(IRElement::text(&original));
                    } else {
                        let normalized = normalize_param_name(&original);
                        if original != normalized {
                            renames.insert(original, normalized.clone());
                        }
                        ir.push(IRElement::text(&normalized));
                    }
                }
            }
        }
        ir.push(IRElement::text(")"));
    } else {
        // Single variable declaration
        // Type
        ir.push(format_expression(&decl.ty));

        // Storage location
        if let Some(storage) = &decl.storage {
            ir.push(IRElement::text(" "));
            ir.push(format_storage_location(storage));
        }

        // Name - normalize local variable names with underscore prefix
        // Exception: single-character names (like loop vars i, j, k) are left as-is
        ir.push(IRElement::text(" "));
        if let Some(name) = &decl.name {
            let original = name.name.clone();
            // Skip normalization for single-character variable names
            if original.len() == 1 {
                ir.push(IRElement::text(&original));
            } else {
                let normalized = normalize_param_name(&original);
                // Add to renames map so subsequent references use the normalized name
                if original != normalized {
                    renames.insert(original, normalized.clone());
                }
                ir.push(IRElement::text(&normalized));
            }
        }
    }

    // Initializer - SoftestLineBreak before Indent means:
    // - If we break after =, the expr is indented
    // - If we don't break (greedy case with HardLineBreaks), no extra indent
    if let Some(init_expr) = init {
        // Format initializer with the updated renames map
        let fmt = |e: &Expression| format_expression_with_renames(e, renames);
        ir.push(IRElement::text(" = "));
        ir.push(IRElement::SoftestLineBreak);
        ir.push(IRElement::indent(vec![fmt(init_expr)]));
    }

    ir.push(IRElement::text(";"));
    vec![IRElement::group(ir)]
}

fn format_for_with_renames(
    init: Option<&Statement>,
    cond: Option<&Expression>,
    update: Option<&Expression>,
    body: Option<&Statement>,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("for (")];

    // Init - may introduce new local variables that affect subsequent expressions
    if let Some(init_stmt) = init {
        ir.extend(format_statement_with_renames(init_stmt, renames));
    } else {
        ir.push(IRElement::text(";"));
    }

    ir.push(IRElement::text(" "));

    // Condition - uses renames (including any from init)
    if let Some(cond_expr) = cond {
        ir.push(format_expression_with_renames(cond_expr, renames));
    }
    ir.push(IRElement::text("; "));

    // Update - uses renames
    if let Some(update_expr) = update {
        ir.push(format_expression_with_renames(update_expr, renames));
    }

    ir.push(IRElement::text(") "));

    // Body - may introduce more local variables
    if let Some(body_stmt) = body {
        ir.extend(format_statement_with_renames(body_stmt, renames));
    } else {
        ir.push(IRElement::text(";"));
    }

    ir
}

fn format_do_while_with_renames(
    body: &Statement,
    cond: &Expression,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("do"), IRElement::text(" ")];

    // Body - may introduce local variables
    ir.extend(format_statement_with_renames(body, renames));
    ir.push(IRElement::text(" while ("));
    // Condition - uses renames (including any from body)
    ir.push(format_expression_with_renames(cond, renames));
    ir.push(IRElement::text(");"));

    ir
}

fn format_return_with_renames(
    val: Option<&Expression>,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);
    let mut ir = vec![IRElement::text("return")];

    if let Some(expr) = val {
        ir.push(IRElement::text(" "));
        ir.push(fmt(expr));
    }

    ir.push(IRElement::text(";"));
    vec![IRElement::group(ir)]
}

fn format_revert_with_renames(
    error: Option<&IdentifierPath>,
    args: &[Expression],
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);
    let mut ir = vec![IRElement::text("revert")];

    if let Some(err) = error {
        ir.push(IRElement::text(" "));
        ir.push(format_identifier_path(err));
    }

    ir.push(IRElement::text("("));

    if !args.is_empty() {
        let mut args_ir = vec![];
        for (i, arg) in args.iter().enumerate() {
            if i > 0 {
                args_ir.push(IRElement::text(", "));
            }
            args_ir.push(fmt(arg));
        }
        ir.push(IRElement::indent(vec![
            IRElement::SoftestLineBreak,
            IRElement::group(args_ir),
        ]));
        ir.push(IRElement::SoftestLineBreak);
    }

    ir.push(IRElement::text(");"));
    vec![IRElement::group(ir)]
}

fn format_revert_named_with_renames(
    error: Option<&IdentifierPath>,
    args: &[NamedArgument],
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);
    let mut ir = vec![IRElement::text("revert")];

    if let Some(err) = error {
        ir.push(IRElement::text(" "));
        ir.push(format_identifier_path(err));
    }

    ir.push(IRElement::text("({"));

    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            ir.push(IRElement::text(", "));
        }
        ir.push(IRElement::text(&arg.name.name));
        ir.push(IRElement::text(": "));
        ir.push(fmt(&arg.expr));
    }

    ir.push(IRElement::text("});"));
    vec![IRElement::group(ir)]
}

fn format_try_with_renames(
    expr: &Expression,
    returns: &[(Loc, Option<Parameter>)],
    clauses: &[CatchClause],
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let fmt = |e: &Expression| format_expression_with_renames(e, renames);
    let mut ir = vec![IRElement::text("try"), IRElement::text(" "), fmt(expr)];

    // Returns
    if !returns.is_empty() {
        ir.push(IRElement::text(" returns ("));
        for (i, (_, param)) in returns.iter().enumerate() {
            if i > 0 {
                ir.push(IRElement::text(", "));
            }
            if let Some(p) = param {
                ir.extend(crate::ir_builder::expressions::format_parameter(
                    p,
                ));
            }
        }
        ir.push(IRElement::text(")"));
    }

    // Catch clauses
    for clause in clauses {
        match clause {
            CatchClause::Simple(_, param, stmt) => {
                ir.push(IRElement::text(" catch "));
                if let Some(p) = param {
                    ir.push(IRElement::text("("));
                    ir.extend(crate::ir_builder::expressions::format_parameter(
                        p,
                    ));
                    ir.push(IRElement::text(")"));
                }
                ir.push(IRElement::text(" "));
                ir.extend(format_statement_with_renames(stmt, renames));
            }
            CatchClause::Named(_, name, param, stmt) => {
                ir.push(IRElement::text(" catch "));
                ir.push(IRElement::text(&name.name));
                ir.push(IRElement::text("("));
                ir.extend(crate::ir_builder::expressions::format_parameter(
                    param,
                ));
                ir.push(IRElement::text(")"));
                ir.push(IRElement::text(" "));
                ir.extend(format_statement_with_renames(stmt, renames));
            }
        }
    }

    ir
}

fn format_assembly(
    loc: &Loc,
    dialect: Option<&StringLiteral>,
    flags: Option<&Vec<StringLiteral>>,
    block: &YulBlock,
) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("assembly")];

    // Add dialect if present
    if let Some(d) = dialect {
        ir.push(IRElement::text(" "));
        ir.push(IRElement::text(&format!("\"{}\"", d.string)));
    }

    // Add flags if present
    if let Some(f) = flags {
        if !f.is_empty() {
            ir.push(IRElement::text(" "));
            ir.push(IRElement::text("("));
            for (i, flag) in f.iter().enumerate() {
                if i > 0 {
                    ir.push(IRElement::text(", "));
                }
                ir.push(IRElement::text(&format!("\"{}\"", flag.string)));
            }
            ir.push(IRElement::text(")"));
        }
    }

    ir.push(IRElement::text(" "));
    ir.extend(format_yul_block(block));

    ir
}

/// Format assembly with collected YUL block and variable renames applied
pub fn format_assembly_with_collected_and_renames(
    _loc: &Loc,
    dialect: Option<&StringLiteral>,
    flags: Option<&Vec<StringLiteral>>,
    collected_yul: &CollectedYulBlock,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("assembly")];

    // Add dialect if present
    if let Some(d) = dialect {
        ir.push(IRElement::text(" "));
        ir.push(IRElement::text(&format!("\"{}\"", d.string)));
    }

    // Add flags if present
    if let Some(f) = flags {
        if !f.is_empty() {
            ir.push(IRElement::text(" "));
            ir.push(IRElement::text("("));
            for (i, flag) in f.iter().enumerate() {
                if i > 0 {
                    ir.push(IRElement::text(", "));
                }
                ir.push(IRElement::text(&format!("\"{}\"", flag.string)));
            }
            ir.push(IRElement::text(")"));
        }
    }

    ir.push(IRElement::text(" "));
    ir.extend(format_collected_yul_block_with_renames(
        collected_yul,
        renames,
    ));

    ir
}

/// Format collected YUL block with variable renames applied
fn format_collected_yul_block_with_renames(
    block: &CollectedYulBlock,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("{")];

    if !block.statements.is_empty() || !block.standalone_comments.is_empty() {
        ir.push(IRElement::HardLineBreak);

        // Build a list of (position, item) for sorting
        let mut items: Vec<(usize, YulBlockItem)> = Vec::new();

        // Add statements with their positions
        for stmt in &block.statements {
            let pos = get_yul_statement_start(&stmt.statement);
            items.push((pos, YulBlockItem::Statement(stmt)));
        }

        // Group consecutive standalone comments and add with position of first
        if !block.standalone_comments.is_empty() {
            let mut comment_groups: Vec<Vec<&Comment>> = Vec::new();
            let mut current_group: Vec<&Comment> = Vec::new();

            for comment in &block.standalone_comments {
                if current_group.is_empty() {
                    current_group.push(comment);
                } else {
                    let last_end = get_loc_start(&current_group.last().unwrap().loc()) + 50;
                    let curr_start = get_loc_start(&comment.loc());
                    if curr_start < last_end + 200 {
                        current_group.push(comment);
                    } else {
                        comment_groups.push(current_group);
                        current_group = vec![comment];
                    }
                }
            }
            if !current_group.is_empty() {
                comment_groups.push(current_group);
            }

            for group in comment_groups {
                let pos = get_loc_start(&group[0].loc());
                items.push((pos, YulBlockItem::StandaloneComments(group)));
            }
        }

        items.sort_by_key(|(pos, _)| *pos);

        let mut content = vec![];
        for (_, item) in items {
            match item {
                YulBlockItem::Statement(stmt) => {
                    if !stmt.leading_comments.is_empty() && !content.is_empty() {
                        content.push(IRElement::HardLineBreak);
                    }
                    content.extend(build_yul_statement_ir_with_renames(stmt, renames));
                    content.push(IRElement::HardLineBreak);
                }
                YulBlockItem::StandaloneComments(comments) => {
                    if !content.is_empty() {
                        content.push(IRElement::HardLineBreak);
                    }
                    content.extend(build_combined_comment(&comments));
                }
            }
        }

        if !content.is_empty() {
            content.pop();
        }

        ir.push(IRElement::indent(content));
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text("}"));
    ir
}

/// Build YUL statement IR with variable renames applied
fn build_yul_statement_ir_with_renames(
    stmt: &CommentedYulStatement,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    with_comments(&stmt.leading_comments, &stmt.trailing_comments, || {
        format_yul_statement_with_renames(&stmt.statement, renames)
    })
}

/// Format a YUL statement with variable renames applied
fn format_yul_statement_with_renames(
    stmt: &YulStatement,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    match stmt {
        YulStatement::Assign(_, vars, expr) => {
            let mut ir = vec![];
            for (i, var) in vars.iter().enumerate() {
                if i > 0 {
                    ir.push(IRElement::text(", "));
                }
                ir.push(format_yul_expression_with_renames(var, renames));
            }
            ir.push(IRElement::text(" "));
            ir.push(IRElement::text(":="));
            ir.push(IRElement::text(" "));
            ir.push(format_yul_expression_with_renames(expr, renames));
            ir
        }
        YulStatement::VariableDeclaration(_, vars, expr) => {
            let mut ir = vec![IRElement::text("let"), IRElement::text(" ")];
            for (i, var) in vars.iter().enumerate() {
                if i > 0 {
                    ir.push(IRElement::text(", "));
                }
                // Normalize variable name (skip single-character names)
                let original = var.id.name.clone();
                let normalized = if original.len() == 1 {
                    original.clone()
                } else {
                    let n = normalize_param_name(&original);
                    if original != n {
                        renames.insert(original, n.clone());
                    }
                    n
                };
                ir.push(IRElement::text(&normalized));
                if let Some(ty) = &var.ty {
                    ir.push(IRElement::text(": "));
                    ir.push(IRElement::text(&ty.name));
                }
            }
            if let Some(e) = expr {
                ir.push(IRElement::text(" "));
                ir.push(IRElement::text(":="));
                ir.push(IRElement::text(" "));
                ir.push(format_yul_expression_with_renames(e, renames));
            }
            ir
        }
        YulStatement::FunctionCall(call) => {
            vec![format_yul_function_call_with_renames(call, renames)]
        }
        YulStatement::If(_, cond, block) => {
            let mut ir = vec![
                IRElement::text("if"),
                IRElement::text(" "),
                format_yul_expression_with_renames(cond, renames),
                IRElement::text(" "),
            ];
            ir.extend(format_yul_block_with_renames(block, renames));
            ir
        }
        YulStatement::Switch(switch) => {
            let mut ir = vec![
                IRElement::text("switch"),
                IRElement::text(" "),
                format_yul_expression_with_renames(&switch.condition, renames),
                IRElement::HardLineBreak,
            ];
            // Process regular cases
            for case in &switch.cases {
                match case {
                    YulSwitchOptions::Case(_, expr, block) => {
                        ir.push(IRElement::text("case"));
                        ir.push(IRElement::text(" "));
                        ir.push(format_yul_expression_with_renames(expr, renames));
                        ir.push(IRElement::text(" "));
                        ir.extend(format_yul_block_with_renames(block, renames));
                        ir.push(IRElement::HardLineBreak);
                    }
                    YulSwitchOptions::Default(_, block) => {
                        ir.push(IRElement::text("default"));
                        ir.push(IRElement::text(" "));
                        ir.extend(format_yul_block_with_renames(block, renames));
                    }
                }
            }
            // Process default case if present
            if let Some(default_case) = &switch.default {
                if let YulSwitchOptions::Default(_, block) = default_case {
                    ir.push(IRElement::text("default"));
                    ir.push(IRElement::text(" "));
                    ir.extend(format_yul_block_with_renames(block, renames));
                }
            }
            ir
        }
        // For other statement types, delegate to the original function
        _ => format_yul_statement(stmt),
    }
}

/// Format a YUL expression with variable renames applied
fn format_yul_expression_with_renames(
    expr: &YulExpression,
    renames: &mut HashMap<String, String>,
) -> IRElement {
    match expr {
        YulExpression::Variable(var) => {
            // Apply rename if this variable has one
            let name = renames.get(&var.name).unwrap_or(&var.name);
            IRElement::text(name)
        }
        YulExpression::FunctionCall(call) => {
            format_yul_function_call_with_renames(call, renames)
        }
        YulExpression::SuffixAccess(_, inner, suffix) => {
            IRElement::group(vec![
                format_yul_expression_with_renames(inner, renames),
                IRElement::text("."),
                IRElement::text(&suffix.name),
            ])
        }
        // For other expressions (literals), delegate to the original function
        _ => format_yul_expression(expr),
    }
}

/// Format a YUL function call with variable renames applied to arguments
fn format_yul_function_call_with_renames(
    call: &YulFunctionCall,
    renames: &mut HashMap<String, String>,
) -> IRElement {
    let mut ir = vec![IRElement::text(&call.id.name), IRElement::text("(")];

    for (i, arg) in call.arguments.iter().enumerate() {
        if i > 0 {
            ir.push(IRElement::text(","));
            ir.push(IRElement::text(" "));
        }
        ir.push(format_yul_expression_with_renames(arg, renames));
    }

    ir.push(IRElement::text(")"));
    IRElement::group(ir)
}

/// Format a YUL block with variable renames applied
fn format_yul_block_with_renames(
    block: &YulBlock,
    renames: &mut HashMap<String, String>,
) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("{")];

    if !block.statements.is_empty() {
        ir.push(IRElement::HardLineBreak);

        let mut stmts = vec![];
        for stmt in &block.statements {
            stmts.extend(format_yul_statement_with_renames(stmt, renames));
            stmts.push(IRElement::HardLineBreak);
        }

        // Remove last line break
        if !stmts.is_empty() {
            stmts.pop();
        }

        ir.push(IRElement::indent(stmts));
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text("}"));
    ir
}

fn format_yul_block(block: &YulBlock) -> Vec<IRElement> {
    let mut ir = vec![IRElement::text("{")];

    if !block.statements.is_empty() {
        ir.push(IRElement::HardLineBreak);

        let mut stmts = vec![];
        for stmt in &block.statements {
            stmts.extend(format_yul_statement(stmt));
            stmts.push(IRElement::HardLineBreak);
        }

        // Remove last line break
        if !stmts.is_empty() {
            stmts.pop();
        }

        ir.push(IRElement::indent(stmts));
        ir.push(IRElement::HardLineBreak);
    }

    ir.push(IRElement::text("}"));
    ir
}

/// Represents either a statement or standalone comments at a position
enum YulBlockItem<'a> {
    Statement(&'a CommentedYulStatement),
    StandaloneComments(Vec<&'a Comment>),
}

/// Combine consecutive comments into a single Comment IR element.
/// The printer will decide how to render it (line comment, wrapped, or block).
fn build_combined_comment(comments: &[&Comment]) -> Vec<IRElement> {
    if comments.is_empty() {
        return vec![];
    }

    // If only one comment, just output it directly
    if comments.len() == 1 {
        return vec![build_comment(comments[0]), IRElement::HardLineBreak];
    }

    // Extract text from each comment and check if any are doc comments
    let mut lines: Vec<String> = Vec::new();
    let mut is_doc = false;

    for comment in comments {
        let text = match comment {
            Comment::Line(_, text) => text
                .trim()
                .strip_prefix("//")
                .unwrap_or(text.trim())
                .trim()
                .to_string(),
            Comment::DocLine(_, text) => {
                is_doc = true;
                text.trim()
                    .strip_prefix("///")
                    .unwrap_or(text.trim())
                    .trim()
                    .to_string()
            }
            Comment::Block(_, text) => {
                let inner = text.trim();
                let inner = inner.strip_prefix("/*").unwrap_or(inner);
                let inner = inner.strip_suffix("*/").unwrap_or(inner);
                inner.trim().to_string()
            }
            Comment::DocBlock(_, text) => {
                is_doc = true;
                let inner = text.trim();
                let inner = inner.strip_prefix("/**").unwrap_or(inner);
                let inner = inner.strip_suffix("*/").unwrap_or(inner);
                inner.trim().to_string()
            }
        };
        if !text.is_empty() {
            lines.push(text);
        }
    }

    if lines.is_empty() {
        return vec![];
    }

    // Combine all text and create a single Comment IR element
    let combined_text = lines.join(" ");
    vec![
        IRElement::comment(combined_text, is_doc),
        IRElement::HardLineBreak,
    ]
}

/// Get the start position of a YUL statement
fn get_yul_statement_start(stmt: &YulStatement) -> usize {
    match stmt {
        YulStatement::Assign(loc, _, _) => get_loc_start(loc),
        YulStatement::VariableDeclaration(loc, _, _) => get_loc_start(loc),
        YulStatement::If(loc, _, _) => get_loc_start(loc),
        YulStatement::For(yul_for) => get_loc_start(&yul_for.loc),
        YulStatement::Switch(yul_switch) => get_loc_start(&yul_switch.loc),
        YulStatement::Leave(loc) => get_loc_start(loc),
        YulStatement::Break(loc) => get_loc_start(loc),
        YulStatement::Continue(loc) => get_loc_start(loc),
        YulStatement::Block(block) => get_loc_start(&block.loc),
        YulStatement::FunctionDefinition(func_def) => get_loc_start(&func_def.loc),
        YulStatement::FunctionCall(func_call) => get_loc_start(&func_call.loc),
        YulStatement::Error(loc) => get_loc_start(loc),
    }
}

fn format_yul_statement(stmt: &YulStatement) -> Vec<IRElement> {
    match stmt {
        YulStatement::Assign(_, vars, expr) => {
            let mut ir = vec![];
            for (i, var) in vars.iter().enumerate() {
                if i > 0 {
                    ir.push(IRElement::text(", "));
                }
                ir.push(format_yul_expression(var));
            }
            ir.push(IRElement::text(" "));
            ir.push(IRElement::text(":="));
            ir.push(IRElement::text(" "));
            ir.push(format_yul_expression(expr));
            ir
        }
        YulStatement::VariableDeclaration(_, vars, expr) => {
            let mut ir = vec![IRElement::text("let"), IRElement::text(" ")];
            for (i, var) in vars.iter().enumerate() {
                if i > 0 {
                    ir.push(IRElement::text(", "));
                }
                ir.push(IRElement::text(&var.id.name));
                if let Some(ty) = &var.ty {
                    ir.push(IRElement::text(": "));
                    ir.push(IRElement::text(&ty.name));
                }
            }
            if let Some(e) = expr {
                ir.push(IRElement::text(" "));
                ir.push(IRElement::text(":="));
                ir.push(IRElement::text(" "));
                ir.push(format_yul_expression(e));
            }
            ir
        }
        YulStatement::FunctionCall(call) => vec![format_yul_function_call(call)],
        YulStatement::If(_, cond, block) => {
            let mut ir = vec![
                IRElement::text("if"),
                IRElement::text(" "),
                format_yul_expression(cond),
                IRElement::text(" "),
            ];
            ir.extend(format_yul_block(block));
            ir
        }
        YulStatement::For(yul_for) => {
            let mut ir = vec![IRElement::text("for"), IRElement::text(" ")];
            ir.extend(format_yul_block(&yul_for.init_block));
            ir.push(IRElement::text(" "));
            ir.push(format_yul_expression(&yul_for.condition));
            ir.push(IRElement::text(" "));
            ir.extend(format_yul_block(&yul_for.post_block));
            ir.push(IRElement::text(" "));
            ir.extend(format_yul_block(&yul_for.execution_block));
            ir
        }
        YulStatement::Switch(switch) => {
            // Check if we have a collected switch in the parent context
            // For now, just format normally
            format_yul_switch(switch)
        }
        YulStatement::Leave(_) => vec![IRElement::text("leave")],
        YulStatement::Break(_) => vec![IRElement::text("break")],
        YulStatement::Continue(_) => vec![IRElement::text("continue")],
        YulStatement::Block(block) => format_yul_block(block),
        YulStatement::FunctionDefinition(def) => format_yul_function_def(def),
        YulStatement::Error(_) => vec![IRElement::text("/* yul error */")],
    }
}

fn format_yul_expression(expr: &YulExpression) -> IRElement {
    match expr {
        YulExpression::BoolLiteral(_, val, _) => {
            IRElement::text(if *val { "true" } else { "false" })
        }
        YulExpression::NumberLiteral(_, val, _, _) => IRElement::text(val),
        YulExpression::StringLiteral(lit, _) => IRElement::text(&format!("\"{}\"", lit.string)),
        YulExpression::HexNumberLiteral(_, val, _) => IRElement::text(val),
        YulExpression::HexStringLiteral(lit, _) => IRElement::text(&format!("hex\"{}\"", lit.hex)),
        YulExpression::Variable(var) => IRElement::text(&var.name),
        YulExpression::FunctionCall(call) => format_yul_function_call(call),
        YulExpression::SuffixAccess(_, expr, suffix) => IRElement::group(vec![
            format_yul_expression(expr),
            IRElement::text("."),
            IRElement::text(&suffix.name),
        ]),
    }
}

fn format_yul_function_call(call: &YulFunctionCall) -> IRElement {
    let mut ir = vec![IRElement::text(&call.id.name), IRElement::text("(")];

    for (i, arg) in call.arguments.iter().enumerate() {
        if i > 0 {
            ir.push(IRElement::text(","));
            ir.push(IRElement::text(" "));
        }
        ir.push(format_yul_expression(arg));
    }

    ir.push(IRElement::text(")"));
    IRElement::group(ir)
}

fn format_yul_switch(switch: &YulSwitch) -> Vec<IRElement> {
    let mut ir = vec![
        IRElement::text("switch"),
        IRElement::text(" "),
        format_yul_expression(&switch.condition),
        IRElement::HardLineBreak,
    ];

    // Process regular cases
    for case in &switch.cases {
        match case {
            YulSwitchOptions::Case(_, expr, block) => {
                ir.push(IRElement::text("case"));
                ir.push(IRElement::text(" "));
                ir.push(format_yul_expression(expr));
                ir.push(IRElement::text(" "));
                ir.extend(format_yul_block(block));
                ir.push(IRElement::HardLineBreak);
            }
            YulSwitchOptions::Default(_, block) => {
                // This shouldn't happen - default should be in the separate field
                ir.push(IRElement::text("default"));
                ir.push(IRElement::text(" "));
                ir.extend(format_yul_block(block));
            }
        }
    }

    // Process default case if present
    if let Some(default_case) = &switch.default {
        match default_case {
            YulSwitchOptions::Default(_, block) => {
                ir.push(IRElement::text("default"));
                ir.push(IRElement::text(" "));
                ir.extend(format_yul_block(block));
            }
            _ => {} // Default case should always be Default variant
        }
    }

    ir
}

fn format_yul_function_def(def: &YulFunctionDefinition) -> Vec<IRElement> {
    let mut ir = vec![
        IRElement::text("function"),
        IRElement::text(" "),
        IRElement::text(&def.id.name),
        IRElement::text("("),
    ];

    for (i, param) in def.params.iter().enumerate() {
        if i > 0 {
            ir.push(IRElement::text(", "));
        }
        ir.push(IRElement::text(&param.id.name));
        if let Some(ty) = &param.ty {
            ir.push(IRElement::text(": "));
            ir.push(IRElement::text(&ty.name));
        }
    }

    ir.push(IRElement::text(")"));

    if !def.returns.is_empty() {
        ir.push(IRElement::text(" -> "));
        for (i, ret) in def.returns.iter().enumerate() {
            if i > 0 {
                ir.push(IRElement::text(", "));
            }
            ir.push(IRElement::text(&ret.id.name));
            if let Some(ty) = &ret.ty {
                ir.push(IRElement::text(": "));
                ir.push(IRElement::text(&ty.name));
            }
        }
    }

    ir.push(IRElement::text(" "));
    ir.extend(format_yul_block(&def.body));

    ir
}
