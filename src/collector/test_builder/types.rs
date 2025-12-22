use super::*;
use crate::collector::model::{CollectedEnum, CollectedStruct, CommentedElement};
use solang_parser::pt::{Identifier, TypeDefinition, VariableDeclaration};

// Struct field expectation
pub struct FieldExpectation {
    name: String,
    ty: Option<String>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl FieldExpectation {
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

    pub fn assert_matches(
        &self,
        field: &CommentedElement<Box<VariableDeclaration>>,
        context: &str,
    ) {
        // Check field name
        let field_name = match &field.element.name {
            Some(name) => &name.name,
            None => panic!("{}: field has no name", context),
        };
        assert_eq!(field_name, &self.name, "{}: field name mismatch", context);

        // Check type if specified
        if let Some(expected_ty) = &self.ty {
            let actual_ty = extract_type_string(&field.element.ty);
            assert_eq!(expected_ty, &actual_ty, "{}: field type mismatch", context);
        }

        // Check comments
        assert_comments(
            &field.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &field.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for FieldExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// Struct expectation
pub struct StructExpectation {
    name: String,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    fields: Vec<FieldExpectation>,
}

impl StructExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            leading_comments: vec![],
            trailing_comments: vec![],
            fields: vec![],
        }
    }

    pub fn field<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(FieldExpectation) -> FieldExpectation,
    {
        self.fields.push(f(FieldExpectation::new(name)));
        self
    }

    pub fn assert_matches(&self, struct_def: &CollectedStruct, context: &str) {
        // Check struct name
        let struct_name = match &struct_def.definition.element.name {
            Some(name) => &name.name,
            None => panic!("{}: struct has no name", context),
        };
        assert_eq!(struct_name, &self.name, "{}: struct name mismatch", context);

        // Check comments
        assert_comments(
            &struct_def.definition.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &struct_def.definition.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check fields
        assert_eq!(
            self.fields.len(),
            struct_def.fields.len(),
            "{}: expected {} fields, found {}",
            context,
            self.fields.len(),
            struct_def.fields.len()
        );

        for (i, field_exp) in self.fields.iter().enumerate() {
            field_exp.assert_matches(&struct_def.fields[i], &format!("{}.field[{}]", context, i));
        }
    }
}

impl CommentExpectation for StructExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// Enum value expectation
pub struct EnumValueExpectation {
    name: String,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl EnumValueExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            leading_comments: vec![],
            trailing_comments: vec![],
        }
    }

    pub fn assert_matches(&self, value: &CommentedElement<Option<Identifier>>, context: &str) {
        // Check enum value name
        let value_name = match &value.element {
            Some(ident) => &ident.name,
            None => panic!("{}: enum value is None", context),
        };
        assert_eq!(
            value_name, &self.name,
            "{}: enum value name mismatch",
            context
        );

        // Check comments
        assert_comments(
            &value.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &value.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for EnumValueExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// Enum expectation
pub struct EnumExpectation {
    name: String,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    values: Vec<EnumValueExpectation>,
}

impl EnumExpectation {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            leading_comments: vec![],
            trailing_comments: vec![],
            values: vec![],
        }
    }

    pub fn value<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(EnumValueExpectation) -> EnumValueExpectation,
    {
        self.values.push(f(EnumValueExpectation::new(name)));
        self
    }

    pub fn assert_matches(&self, enum_def: &CollectedEnum, context: &str) {
        // Check enum name
        let enum_name = match &enum_def.definition.element.name {
            Some(name) => &name.name,
            None => panic!("{}: enum has no name", context),
        };
        assert_eq!(enum_name, &self.name, "{}: enum name mismatch", context);

        // Check comments
        assert_comments(
            &enum_def.definition.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &enum_def.definition.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check values
        assert_eq!(
            self.values.len(),
            enum_def.values.len(),
            "{}: expected {} values, found {}",
            context,
            self.values.len(),
            enum_def.values.len()
        );

        for (i, value_exp) in self.values.iter().enumerate() {
            value_exp.assert_matches(&enum_def.values[i], &format!("{}.value[{}]", context, i));
        }
    }
}

impl CommentExpectation for EnumExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// Type alias expectation
pub struct TypeExpectation {
    name: String,
    ty: Option<String>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl TypeExpectation {
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

    pub fn assert_matches(&self, type_def: &CommentedElement<Box<TypeDefinition>>, context: &str) {
        // Check type alias name
        assert_eq!(
            type_def.element.name.name, self.name,
            "{}: type alias name mismatch",
            context
        );

        // Check type if specified
        if let Some(expected_ty) = &self.ty {
            let actual_ty = extract_type_string(&type_def.element.ty);
            assert_eq!(
                expected_ty, &actual_ty,
                "{}: type alias type mismatch",
                context
            );
        }

        // Check comments
        assert_comments(
            &type_def.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &type_def.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for TypeExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}
