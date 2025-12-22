// This module contains the ExpectationBuilder for creating test expectations

use super::assert_comment_text;
use super::events_errors::ErrorExpectation;
use super::variables::VariableExpectation;
use crate::collector::model::CollectedElements;
use crate::collector::test_builder::{
    ContractExpectation, EnumExpectation, FunctionExpectation, ImportExpectation,
    PragmaExpectation, StructExpectation, TypeExpectation, UsingExpectation,
};

/// Builder for creating test expectations that can be matched against CollectedElements
pub struct ExpectationBuilder {
    pragmas: Vec<PragmaExpectation>,
    imports: Vec<ImportExpectation>,
    contracts: Vec<ContractExpectation>,
    structs: Vec<StructExpectation>,
    enums: Vec<EnumExpectation>,
    functions: Vec<FunctionExpectation>,
    variables: Vec<VariableExpectation>,
    errors: Vec<ErrorExpectation>,
    types: Vec<TypeExpectation>,
    using_directives: Vec<UsingExpectation>,
    standalone_comments: Vec<String>,
}

impl ExpectationBuilder {
    pub fn new() -> Self {
        Self {
            pragmas: vec![],
            imports: vec![],
            contracts: vec![],
            structs: vec![],
            enums: vec![],
            functions: vec![],
            variables: vec![],
            errors: vec![],
            types: vec![],
            using_directives: vec![],
            standalone_comments: vec![],
        }
    }

    pub fn pragma<F>(mut self, f: F) -> Self
    where
        F: FnOnce(PragmaExpectation) -> PragmaExpectation,
    {
        self.pragmas.push(f(PragmaExpectation::new()));
        self
    }

    pub fn import<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ImportExpectation) -> ImportExpectation,
    {
        self.imports.push(f(ImportExpectation::new()));
        self
    }

    pub fn contract<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(ContractExpectation) -> ContractExpectation,
    {
        self.contracts.push(f(ContractExpectation::new(name)));
        self
    }

    pub fn struct_def<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(StructExpectation) -> StructExpectation,
    {
        self.structs.push(f(StructExpectation::new(name)));
        self
    }

    pub fn enum_def<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(EnumExpectation) -> EnumExpectation,
    {
        self.enums.push(f(EnumExpectation::new(name)));
        self
    }

    pub fn function<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(FunctionExpectation) -> FunctionExpectation,
    {
        self.functions.push(f(FunctionExpectation::new(name)));
        self
    }

    pub fn variable<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(VariableExpectation) -> VariableExpectation,
    {
        self.variables.push(f(VariableExpectation::new(name)));
        self
    }

    pub fn error<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(ErrorExpectation) -> ErrorExpectation,
    {
        self.errors.push(f(ErrorExpectation::new(name)));
        self
    }

    pub fn type_def<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(TypeExpectation) -> TypeExpectation,
    {
        self.types.push(f(TypeExpectation::new(name)));
        self
    }

    pub fn using<F>(mut self, f: F) -> Self
    where
        F: FnOnce(UsingExpectation) -> UsingExpectation,
    {
        self.using_directives.push(f(UsingExpectation::new()));
        self
    }

    pub fn standalone_comment(mut self, text: &str) -> Self {
        self.standalone_comments.push(text.to_string());
        self
    }

    /// Assert that the collected elements match the expectations
    pub fn assert_matches(&self, collected: &CollectedElements) {
        // Check counts
        assert_eq!(
            self.pragmas.len(),
            collected.pragmas.len(),
            "Expected {} pragmas, found {}",
            self.pragmas.len(),
            collected.pragmas.len()
        );
        assert_eq!(
            self.contracts.len(),
            collected.contracts.len(),
            "Expected {} contracts, found {}",
            self.contracts.len(),
            collected.contracts.len()
        );
        assert_eq!(
            self.structs.len(),
            collected.structs.len(),
            "Expected {} structs, found {}",
            self.structs.len(),
            collected.structs.len()
        );
        assert_eq!(
            self.enums.len(),
            collected.enums.len(),
            "Expected {} enums, found {}",
            self.enums.len(),
            collected.enums.len()
        );
        assert_eq!(
            self.functions.len(),
            collected.functions.len(),
            "Expected {} functions, found {}",
            self.functions.len(),
            collected.functions.len()
        );
        assert_eq!(
            self.variables.len(),
            collected.variables.len(),
            "Expected {} variables, found {}",
            self.variables.len(),
            collected.variables.len()
        );
        assert_eq!(
            self.errors.len(),
            collected.errors.len(),
            "Expected {} errors, found {}",
            self.errors.len(),
            collected.errors.len()
        );
        assert_eq!(
            self.types.len(),
            collected.types.len(),
            "Expected {} types, found {}",
            self.types.len(),
            collected.types.len()
        );
        assert_eq!(
            self.imports.len(),
            collected.imports.len(),
            "Expected {} imports, found {}",
            self.imports.len(),
            collected.imports.len()
        );
        assert_eq!(
            self.using_directives.len(),
            collected.using_directives.len(),
            "Expected {} using directives, found {}",
            self.using_directives.len(),
            collected.using_directives.len()
        );
        assert_eq!(
            self.standalone_comments.len(),
            collected.standalone_comments.len(),
            "Expected {} standalone comments, found {}",
            self.standalone_comments.len(),
            collected.standalone_comments.len()
        );

        // Check each element
        for (i, pragma_exp) in self.pragmas.iter().enumerate() {
            pragma_exp.assert_matches(&collected.pragmas[i], &format!("pragma[{}]", i));
        }

        for (i, contract_exp) in self.contracts.iter().enumerate() {
            contract_exp.assert_matches(&collected.contracts[i], &format!("contract[{}]", i));
        }

        for (i, struct_exp) in self.structs.iter().enumerate() {
            struct_exp.assert_matches(&collected.structs[i], &format!("struct[{}]", i));
        }

        for (i, enum_exp) in self.enums.iter().enumerate() {
            enum_exp.assert_matches(&collected.enums[i], &format!("enum[{}]", i));
        }

        for (i, func_exp) in self.functions.iter().enumerate() {
            func_exp.assert_matches(&collected.functions[i], &format!("function[{}]", i));
        }

        for (i, var_exp) in self.variables.iter().enumerate() {
            var_exp.assert_matches(&collected.variables[i], &format!("variable[{}]", i));
        }

        for (i, error_exp) in self.errors.iter().enumerate() {
            error_exp.assert_matches(&collected.errors[i], &format!("error[{}]", i));
        }

        for (i, type_exp) in self.types.iter().enumerate() {
            type_exp.assert_matches(&collected.types[i], &format!("type[{}]", i));
        }

        for (i, import_exp) in self.imports.iter().enumerate() {
            import_exp.assert_matches(&collected.imports[i], &format!("import[{}]", i));
        }

        for (i, using_exp) in self.using_directives.iter().enumerate() {
            using_exp.assert_matches(&collected.using_directives[i], &format!("using[{}]", i));
        }

        for (i, expected_comment) in self.standalone_comments.iter().enumerate() {
            assert_comment_text(
                &collected.standalone_comments[i],
                expected_comment,
                &format!("standalone_comment[{}]", i),
            );
        }
    }
}
