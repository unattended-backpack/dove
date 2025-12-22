use super::*;
use crate::collector::model::{CollectedFunction, CommentedStatement, FunctionSignatureComments};
use solang_parser::pt::Comment;
use crate::collector::model::TypeExpressionComments;
use solang_parser::pt::{
    FunctionAttribute, FunctionTy, Mutability, Parameter, StorageLocation, Visibility, Loc,
};
use super::variables::{TypeCommentsExpectation, TypeCommentsBuilder};


// Function parameter expectation
pub struct FunctionParameterExpectation {
    name: String,
    ty: Option<String>,
    storage_location: Option<String>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    type_comments: Option<TypeCommentsExpectation>,
}

impl FunctionParameterExpectation {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: None,
            storage_location: None,
            leading_comments: vec![],
            trailing_comments: vec![],
            type_comments: None,
        }
    }

    pub fn ty(mut self, ty: &str) -> Self {
        self.ty = Some(ty.to_string());
        self
    }

    pub fn storage_location(mut self, location: &str) -> Self {
        self.storage_location = Some(location.to_string());
        self
    }

    pub fn storage(mut self, location: &str) -> Self {
        self.storage_location = Some(location.to_string());
        self
    }

    pub fn leading(mut self, comment: &str) -> Self {
        self.leading_comments.push(comment.to_string());
        self
    }

    pub fn trailing(mut self, comment: &str) -> Self {
        self.trailing_comments.push(comment.to_string());
        self
    }

    pub fn type_comments<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TypeCommentsBuilder) -> TypeCommentsBuilder,
    {
        let builder = TypeCommentsBuilder::new();
        let built = f(builder);
        self.type_comments = Some(built.build());
        self
    }

    pub(crate) fn assert_matches(&self, param: &(Loc, Option<Parameter>), context: &str) {
        let param_data = match &param.1 {
            Some(p) => p,
            None => panic!("{}: parameter is None", context),
        };

        // Check parameter name
        let param_name = match &param_data.name {
            Some(name) => &name.name,
            None => "", // Unnamed parameter has empty name
        };
        assert_eq!(
            param_name, &self.name,
            "{}: parameter name mismatch",
            context
        );

        // Check type if specified
        if let Some(expected_ty) = &self.ty {
            // Extract the actual type using the helper function
            let actual_ty = extract_type_string(&param_data.ty);

            assert_eq!(
                expected_ty, &actual_ty,
                "{}: parameter type mismatch",
                context
            );
        }

        // Check storage location if specified
        if let Some(expected_loc) = &self.storage_location {
            let actual_loc = match &param_data.storage {
                Some(loc) => match loc {
                    StorageLocation::Memory(_) => "memory",
                    StorageLocation::Storage(_) => "storage",
                    StorageLocation::Calldata(_) => "calldata",
                },
                None => "",
            };

            if !actual_loc.is_empty() {
                assert_eq!(
                    expected_loc, actual_loc,
                    "{}: parameter storage location mismatch",
                    context
                );
            }
        }

        // Note: Parameters don't have leading/trailing comments in the current collector
        // They might have them in the AST but they're not collected separately
    }
}

impl CommentExpectation for FunctionParameterExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// Function expectations with body statements
pub struct FunctionExpectation {
    name: String,
    visibility: Option<String>,
    mutability: Option<String>,
    is_virtual: Option<bool>,
    is_override: Option<bool>,
    is_modifier: Option<bool>, // True if this is a modifier definition
    modifiers: Vec<String>,
    parameters: Vec<FunctionParameterExpectation>,
    returns: Vec<FunctionParameterExpectation>, // Return parameters
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    body_statements: Vec<StatementExpectation>,
    body_standalone_comments: Vec<String>,
}

impl FunctionExpectation {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            visibility: None,
            mutability: None,
            is_virtual: None,
            is_override: None,
            is_modifier: None,
            modifiers: vec![],
            parameters: vec![],
            returns: vec![],
            leading_comments: vec![],
            trailing_comments: vec![],
            body_statements: vec![],
            body_standalone_comments: vec![],
        }
    }

    pub fn visibility(mut self, visibility: &str) -> Self {
        self.visibility = Some(visibility.to_string());
        self
    }

    pub fn mutability(mut self, mutability: &str) -> Self {
        self.mutability = Some(mutability.to_string());
        self
    }

    pub fn is_virtual(mut self) -> Self {
        self.is_virtual = Some(true);
        self
    }

    pub fn is_override(mut self) -> Self {
        self.is_override = Some(true);
        self
    }

    pub fn overrides(mut self, bases: &[&str]) -> Self {
        // For now, just mark as override - we don't track specific bases in the expectation
        self.is_override = Some(true);
        self
    }

    pub fn returns(mut self, return_type: &str) -> Self {
        let mut return_param = FunctionParameterExpectation::new("");
        return_param.ty = Some(return_type.to_string());
        self.returns.push(return_param);
        self
    }

    pub fn returns_with_comment(mut self, return_type: &str, storage: &str, comment: &str) -> Self {
        let mut return_param = FunctionParameterExpectation::new("");
        return_param.ty = Some(return_type.to_string());
        return_param.storage_location = Some(storage.to_string());
        return_param.leading_comments.push(comment.to_string());
        self.returns.push(return_param);
        self
    }

    pub fn modifier_with_args(mut self, name: &str, _args: &[&str]) -> Self {
        self.modifiers.push(name.to_string());
        self
    }

    pub fn is_modifier(mut self) -> Self {
        self.is_modifier = Some(true);
        self
    }

    pub fn modifier(mut self, modifier: &str) -> Self {
        self.modifiers.push(modifier.to_string());
        self
    }

    pub fn parameter<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(FunctionParameterExpectation) -> FunctionParameterExpectation,
    {
        self.parameters
            .push(f(FunctionParameterExpectation::new(name)));
        self
    }

    pub fn return_param<F>(mut self, f: F) -> Self
    where
        F: FnOnce(FunctionParameterExpectation) -> FunctionParameterExpectation,
    {
        self.returns.push(f(FunctionParameterExpectation::new("")));
        self
    }

    pub fn body_statement<F>(mut self, f: F) -> Self
    where
        F: FnOnce(StatementExpectation) -> StatementExpectation,
    {
        self.body_statements.push(f(StatementExpectation::new()));
        self
    }

    pub fn body_standalone_comment(mut self, comment: &str) -> Self {
        self.body_standalone_comments.push(comment.to_string());
        self
    }

    pub(crate) fn assert_matches(&self, func: &CollectedFunction, context: &str) {
        // Check function name
        let func_name = match &func.definition.element.name {
            Some(name) => &name.name,
            None => "", // Constructor has no name
        };
        assert_eq!(func_name, &self.name, "{}: function name mismatch", context);

        // Check visibility if specified
        if let Some(expected_vis) = &self.visibility {
            let mut found_visibility = None;
            for attr in &func.definition.element.attributes {
                if let FunctionAttribute::Visibility(vis) = attr {
                    found_visibility = Some(format_visibility(vis));
                    break;
                }
            }

            if let Some(actual_vis) = found_visibility {
                assert_eq!(
                    expected_vis, actual_vis,
                    "{}: function visibility mismatch",
                    context
                );
            } else {
                panic!(
                    "{}: expected visibility '{}' but function has no visibility modifier",
                    context, expected_vis
                );
            }
        }

        // Check mutability if specified
        if let Some(expected_mut) = &self.mutability {
            let mut found_mutability = None;
            for attr in &func.definition.element.attributes {
                if let FunctionAttribute::Mutability(mutability) = attr {
                    found_mutability = Some(format_mutability(mutability));
                    break;
                }
            }

            if let Some(actual_mut) = found_mutability {
                assert_eq!(
                    expected_mut, actual_mut,
                    "{}: function mutability mismatch",
                    context
                );
            } else if expected_mut != "nonpayable" {
                // nonpayable is the default, so it's ok if not specified
                panic!(
                    "{}: expected mutability '{}' but function has no mutability modifier",
                    context, expected_mut
                );
            }
        }

        // Check virtual if specified
        if let Some(expected_virtual) = self.is_virtual {
            let is_virtual = func
                .definition
                .element
                .attributes
                .iter()
                .any(|attr| matches!(attr, FunctionAttribute::Virtual(_)));
            assert_eq!(
                expected_virtual, is_virtual,
                "{}: function virtual attribute mismatch",
                context
            );
        }

        // Check override if specified
        if let Some(expected_override) = self.is_override {
            let is_override = func
                .definition
                .element
                .attributes
                .iter()
                .any(|attr| matches!(attr, FunctionAttribute::Override(_, _)));
            assert_eq!(
                expected_override, is_override,
                "{}: function override attribute mismatch",
                context
            );
        }

        // Check if it's a modifier definition
        if let Some(expected_is_modifier) = self.is_modifier {
            let is_modifier = func.definition.element.ty == FunctionTy::Modifier;
            assert_eq!(
                expected_is_modifier,
                is_modifier,
                "{}: expected {} but found {}",
                context,
                if expected_is_modifier {
                    "modifier"
                } else {
                    "function"
                },
                if is_modifier { "modifier" } else { "function" }
            );
        }

        // Check modifiers
        if !self.modifiers.is_empty() {
            let mut actual_modifiers = Vec::new();
            for attr in &func.definition.element.attributes {
                if let FunctionAttribute::BaseOrModifier(_, base) = attr {
                    // IdentifierPath can have multiple segments, but for modifiers we typically just have one
                    if let Some(first_segment) = base.name.identifiers.first() {
                        actual_modifiers.push(&first_segment.name);
                    }
                }
            }

            assert_eq!(
                self.modifiers.len(),
                actual_modifiers.len(),
                "{}: expected {} modifiers, found {}",
                context,
                self.modifiers.len(),
                actual_modifiers.len()
            );

            for (i, expected_mod) in self.modifiers.iter().enumerate() {
                assert_eq!(
                    expected_mod, actual_modifiers[i],
                    "{}: modifier[{}] mismatch",
                    context, i
                );
            }
        }

        // Check parameters
        if !self.parameters.is_empty() {
            let actual_params = &func.definition.element.params;
            assert_eq!(
                self.parameters.len(),
                actual_params.len(),
                "{}: expected {} parameters, found {}",
                context,
                self.parameters.len(),
                actual_params.len()
            );

            for (i, expected_param) in self.parameters.iter().enumerate() {
                expected_param
                    .assert_matches(&actual_params[i], &format!("{}.parameter[{}]", context, i));
            }
        }

        // Check comments
        assert_comments(
            &func.definition.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &func.definition.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check body statements
        assert_eq!(
            self.body_statements.len(),
            func.body_statements.len(),
            "{}: expected {} body statements, found {}",
            context,
            self.body_statements.len(),
            func.body_statements.len()
        );

        for (i, stmt_exp) in self.body_statements.iter().enumerate() {
            stmt_exp.assert_matches(
                &func.body_statements[i],
                &format!("{}.statement[{}]", context, i),
            );
        }

        // Check standalone comments
        assert_eq!(
            self.body_standalone_comments.len(),
            func.body_standalone_comments.len(),
            "{}: expected {} standalone comments, found {}",
            context,
            self.body_standalone_comments.len(),
            func.body_standalone_comments.len()
        );

        for (i, expected_comment) in self.body_standalone_comments.iter().enumerate() {
            assert_comment_text(
                &func.body_standalone_comments[i],
                expected_comment,
                &format!("{}.standalone[{}]", context, i),
            );
        }

        // Check signature comments if present
        if let Some(sig_comments) = &func.signature_comments {
            // Check parameter comments
            for (i, param_exp) in self.parameters.iter().enumerate() {
                if i < sig_comments.parameters.len() {
                    let param_comments = &sig_comments.parameters[i];
                    assert_comments(
                        &param_comments.leading,
                        &param_exp.leading_comments,
                        &format!("{}.param[{}].leading", context, i),
                    );
                    assert_comments(
                        &param_comments.trailing,
                        &param_exp.trailing_comments,
                        &format!("{}.param[{}].trailing", context, i),
                    );
                    // Check type comments if expected
                    if let Some(expected_type_comments) = &param_exp.type_comments {
                        if let Some(actual_type_comments) = &param_comments.type_comments {
                            expected_type_comments.assert_matches(
                                actual_type_comments,
                                &format!("{}.param[{}].type_comments", context, i),
                            );
                        } else {
                            panic!(
                                "{}.param[{}]: expected type comments but found none",
                                context, i
                            );
                        }
                    }
                }
            }

            // Check return comments
            for (i, return_exp) in self.returns.iter().enumerate() {
                if i < sig_comments.returns.len() {
                    let return_comments = &sig_comments.returns[i];
                    assert_comments(
                        &return_comments.leading,
                        &return_exp.leading_comments,
                        &format!("{}.return[{}].leading", context, i),
                    );
                    assert_comments(
                        &return_comments.trailing,
                        &return_exp.trailing_comments,
                        &format!("{}.return[{}].trailing", context, i),
                    );
                    // Check type comments if expected
                    if let Some(expected_type_comments) = &return_exp.type_comments {
                        if let Some(actual_type_comments) = &return_comments.type_comments {
                            expected_type_comments.assert_matches(
                                actual_type_comments,
                                &format!("{}.return[{}].type_comments", context, i),
                            );
                        } else {
                            panic!(
                                "{}.return[{}]: expected type comments but found none",
                                context, i
                            );
                        }
                    }
                }
            }
        }
    }
}

impl CommentExpectation for FunctionExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}
