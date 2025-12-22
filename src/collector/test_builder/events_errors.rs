use super::{assert_comments, extract_type_string, CommentExpectation};
use crate::collector::model::*;
use solang_parser::pt::{ErrorParameter, EventParameter};

pub struct ErrorExpectation {
    name: String,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    parameters: Vec<ErrorParameterExpectation>,
}

impl ErrorExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            leading_comments: vec![],
            trailing_comments: vec![],
            parameters: vec![],
        }
    }

    pub fn parameter<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(ErrorParameterExpectation) -> ErrorParameterExpectation,
    {
        self.parameters
            .push(f(ErrorParameterExpectation::new(name)));
        self
    }

    pub fn assert_matches(&self, error: &CollectedError, context: &str) {
        // Check error name
        let error_name = match &error.definition.element.name {
            Some(name) => &name.name,
            None => panic!("{}: error has no name", context),
        };
        assert_eq!(error_name, &self.name, "{}: error name mismatch", context);

        // Check comments
        assert_comments(
            &error.definition.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &error.definition.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check parameters
        assert_eq!(
            self.parameters.len(),
            error.parameters.len(),
            "{}: expected {} parameters, found {}",
            context,
            self.parameters.len(),
            error.parameters.len()
        );

        for (i, param_exp) in self.parameters.iter().enumerate() {
            param_exp.assert_matches(
                &error.parameters[i],
                &format!("{}.parameter[{}]", context, i),
            );
        }
    }
}

impl CommentExpectation for ErrorExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

pub struct ErrorParameterExpectation {
    name: String,
    ty: Option<String>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl ErrorParameterExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: None,
            leading_comments: vec![],
            trailing_comments: vec![],
        }
    }

    pub fn ty(mut self, ty: &str) -> Self {
        self.ty = Some(ty.to_string());
        self
    }

    pub fn assert_matches(&self, param: &CommentedElement<Box<ErrorParameter>>, context: &str) {
        // Check parameter name
        let param_name = match &param.element.name {
            Some(name) => &name.name,
            None => panic!("{}: error parameter has no name", context),
        };
        assert_eq!(
            param_name, &self.name,
            "{}: error parameter name mismatch",
            context
        );

        // Check type if specified
        if let Some(expected_ty) = &self.ty {
            let actual_ty = extract_type_string(&param.element.ty);
            assert_eq!(
                expected_ty, &actual_ty,
                "{}: error parameter type mismatch",
                context
            );
        }

        // Check comments
        assert_comments(
            &param.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &param.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for ErrorParameterExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

pub struct EventExpectation {
    name: String,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    parameters: Vec<EventParameterExpectation>,
}

impl EventExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            leading_comments: vec![],
            trailing_comments: vec![],
            parameters: vec![],
        }
    }

    pub fn parameter<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(EventParameterExpectation) -> EventParameterExpectation,
    {
        self.parameters
            .push(f(EventParameterExpectation::new(name)));
        self
    }

    pub fn assert_matches(&self, event: &CollectedEvent, context: &str) {
        // Check event name
        let event_name = match &event.definition.element.name {
            Some(name) => &name.name,
            None => panic!("{}: event has no name", context),
        };
        assert_eq!(event_name, &self.name, "{}: event name mismatch", context);

        // Check comments
        assert_comments(
            &event.definition.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &event.definition.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check parameters
        assert_eq!(
            self.parameters.len(),
            event.parameters.len(),
            "{}: expected {} parameters, found {}",
            context,
            self.parameters.len(),
            event.parameters.len()
        );

        for (i, param_exp) in self.parameters.iter().enumerate() {
            param_exp.assert_matches(
                &event.parameters[i],
                &format!("{}.parameter[{}]", context, i),
            );
        }
    }
}

impl CommentExpectation for EventExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

pub struct EventParameterExpectation {
    name: String,
    ty: Option<String>,
    indexed: Option<bool>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl EventParameterExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ty: None,
            indexed: None,
            leading_comments: vec![],
            trailing_comments: vec![],
        }
    }

    pub fn ty(mut self, ty: &str) -> Self {
        self.ty = Some(ty.to_string());
        self
    }

    pub fn indexed(mut self) -> Self {
        self.indexed = Some(true);
        self
    }

    pub fn assert_matches(&self, param: &CommentedElement<Box<EventParameter>>, context: &str) {
        // Check parameter name
        let param_name = match &param.element.name {
            Some(name) => &name.name,
            None => panic!("{}: event parameter has no name", context),
        };
        assert_eq!(
            param_name, &self.name,
            "{}: event parameter name mismatch",
            context
        );

        // Check type if specified
        if let Some(expected_ty) = &self.ty {
            let actual_ty = extract_type_string(&param.element.ty);
            assert_eq!(
                expected_ty, &actual_ty,
                "{}: event parameter type mismatch",
                context
            );
        }

        // Check indexed if specified
        if let Some(expected_indexed) = self.indexed {
            assert_eq!(
                expected_indexed, param.element.indexed,
                "{}: event parameter indexed mismatch",
                context
            );
        }

        // Check comments
        assert_comments(
            &param.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &param.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for EventParameterExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}
