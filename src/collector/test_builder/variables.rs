use super::*;
use crate::collector::model::*;
use solang_parser::pt::{
    Expression, Type, VariableAttribute, VariableDefinition, Visibility,
};

/// Helper function for formatting visibility
fn format_visibility(vis: &Visibility) -> &'static str {
    match vis {
        Visibility::Public(_) => "public",
        Visibility::Private(_) => "private",
        Visibility::Internal(_) => "internal",
        Visibility::External(_) => "external",
    }
}

// Variable expectations
pub struct VariableExpectation {
    name: String,
    ty: Option<String>,
    visibility: Option<String>,
    is_constant: Option<bool>,
    is_immutable: Option<bool>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    type_comments: Option<TypeCommentsExpectation>,
}

#[derive(Debug, Clone)]
pub struct TypeCommentsExpectation {
    key_leading: Vec<String>,
    key_trailing: Vec<String>,
    value_leading: Vec<String>,
    value_trailing: Vec<String>,
}

pub struct TypeCommentsBuilder {
    key_leading: Vec<String>,
    key_trailing: Vec<String>,
    value_leading: Vec<String>,
    value_trailing: Vec<String>,
}

impl TypeCommentsBuilder {
    pub fn new() -> Self {
        Self {
            key_leading: vec![],
            key_trailing: vec![],
            value_leading: vec![],
            value_trailing: vec![],
        }
    }

    pub fn key_leading(mut self, comment: &str) -> Self {
        self.key_leading.push(comment.to_string());
        self
    }

    pub fn key_trailing(mut self, comment: &str) -> Self {
        self.key_trailing.push(comment.to_string());
        self
    }

    pub fn value_leading(mut self, comment: &str) -> Self {
        self.value_leading.push(comment.to_string());
        self
    }

    pub fn value_trailing(mut self, comment: &str) -> Self {
        self.value_trailing.push(comment.to_string());
        self
    }

    pub fn build(self) -> TypeCommentsExpectation {
        TypeCommentsExpectation {
            key_leading: self.key_leading,
            key_trailing: self.key_trailing,
            value_leading: self.value_leading,
            value_trailing: self.value_trailing,
        }
    }
}

impl TypeCommentsExpectation {
    pub fn assert_matches(&self, actual: &TypeExpressionComments, context: &str) {
        // Check key leading comments
        assert_comments(
            &actual.key_leading_comments,
            &self.key_leading,
            &format!("{}.key_leading", context),
        );

        // Check key trailing comments
        assert_comments(
            &actual.key_trailing_comments,
            &self.key_trailing,
            &format!("{}.key_trailing", context),
        );

        // Check value leading comments
        assert_comments(
            &actual.value_leading_comments,
            &self.value_leading,
            &format!("{}.value_leading", context),
        );

        // Check value trailing comments
        assert_comments(
            &actual.value_trailing_comments,
            &self.value_trailing,
            &format!("{}.value_trailing", context),
        );
    }
}

impl VariableExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: None,
            visibility: None,
            is_constant: None,
            is_immutable: None,
            leading_comments: vec![],
            trailing_comments: vec![],
            type_comments: None,
        }
    }

    pub fn ty(mut self, ty: &str) -> Self {
        self.ty = Some(ty.to_string());
        self
    }

    pub fn visibility(mut self, visibility: &str) -> Self {
        self.visibility = Some(visibility.to_string());
        self
    }

    pub fn constant(mut self) -> Self {
        self.is_constant = Some(true);
        self
    }

    pub fn immutable(mut self) -> Self {
        self.is_immutable = Some(true);
        self
    }

    pub fn is_virtual(mut self) -> Self {
        // TODO: Track virtual state variables if needed
        self
    }

    pub fn is_override(mut self) -> Self {
        // TODO: Track override state variables if needed
        self
    }

    pub fn initial_value(mut self, _value: &str) -> Self {
        // TODO: Track initial value if needed
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

    pub fn assert_matches(&self, var: &CommentedElement<Box<VariableDefinition>>, context: &str) {
        // Check variable name
        let var_name = match &var.element.name {
            Some(name) => &name.name,
            None => panic!("{}: variable has no name", context),
        };
        assert_eq!(var_name, &self.name, "{}: variable name mismatch", context);

        // Check type if specified
        if let Some(expected_ty) = &self.ty {
            // Extract the actual type using the helper function
            let actual_ty = extract_type_string(&var.element.ty);

            assert_eq!(
                expected_ty, &actual_ty,
                "{}: variable type mismatch",
                context
            );
        }

        // Check visibility if specified
        if let Some(expected_vis) = &self.visibility {
            let mut found_visibility = None;
            for attr in &var.element.attrs {
                if let VariableAttribute::Visibility(vis) = attr {
                    found_visibility = Some(format_visibility(vis));
                    break;
                }
            }

            if let Some(actual_vis) = found_visibility {
                assert_eq!(
                    expected_vis, actual_vis,
                    "{}: variable visibility mismatch",
                    context
                );
            } else {
                panic!(
                    "{}: expected visibility '{}' but variable has no visibility modifier",
                    context, expected_vis
                );
            }
        }

        // Check constant attribute if specified
        if let Some(expected_constant) = self.is_constant {
            let is_constant = var
                .element
                .attrs
                .iter()
                .any(|attr| matches!(attr, VariableAttribute::Constant(_)));
            assert_eq!(
                expected_constant, is_constant,
                "{}: variable constant attribute mismatch",
                context
            );
        }

        // Check immutable attribute if specified
        if let Some(expected_immutable) = self.is_immutable {
            let is_immutable = var
                .element
                .attrs
                .iter()
                .any(|attr| matches!(attr, VariableAttribute::Immutable(_)));
            assert_eq!(
                expected_immutable, is_immutable,
                "{}: variable immutable attribute mismatch",
                context
            );
        }

        // Check comments
        assert_comments(
            &var.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &var.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check type comments if specified
        if let Some(expected_type_comments) = &self.type_comments {
            if let Some(actual_type_comments) = &var.type_comments {
                assert_comments(
                    &actual_type_comments.key_leading_comments,
                    &expected_type_comments.key_leading,
                    &format!("{}.type_comments.key_leading", context),
                );
                assert_comments(
                    &actual_type_comments.key_trailing_comments,
                    &expected_type_comments.key_trailing,
                    &format!("{}.type_comments.key_trailing", context),
                );
                assert_comments(
                    &actual_type_comments.value_leading_comments,
                    &expected_type_comments.value_leading,
                    &format!("{}.type_comments.value_leading", context),
                );
                assert_comments(
                    &actual_type_comments.value_trailing_comments,
                    &expected_type_comments.value_trailing,
                    &format!("{}.type_comments.value_trailing", context),
                );
            } else {
                panic!("{}: expected type comments but found none", context);
            }
        }
    }
}

impl CommentExpectation for VariableExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}