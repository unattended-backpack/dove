//! Top-level expectation types for pragma, contract, import, and using directives
//!
//! This module contains expectation builders for top-level Solidity constructs that appear
//! at the file scope, including pragma directives, import statements, using directives,
//! and contract definitions.

use crate::collector::model::*;
use solang_parser::pt::{ContractTy, Import, PragmaDirective, Using, UsingList};

// Import helper functions from parent module
use super::events_errors::{ErrorExpectation, EventExpectation};
use super::variables::VariableExpectation;
use super::{
    assert_comment_text, assert_comments, extract_import_path, extract_type_string,
    format_version_requirement, CommentExpectation, FunctionExpectation,
};
use super::{EnumExpectation, StructExpectation, TypeExpectation};

/// Expectation builder for pragma directives
pub struct PragmaExpectation {
    content: Option<String>,
    version: Option<String>,
    string_value: Option<String>,
    second_identifier: Option<String>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl PragmaExpectation {
    pub(crate) fn new() -> Self {
        Self {
            content: None,
            version: None,
            string_value: None,
            second_identifier: None,
            leading_comments: vec![],
            trailing_comments: vec![],
        }
    }

    pub fn content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    pub fn string_value(mut self, value: &str) -> Self {
        self.string_value = Some(value.to_string());
        self
    }

    pub fn second_identifier(mut self, ident: &str) -> Self {
        self.second_identifier = Some(ident.to_string());
        self
    }

    pub(crate) fn assert_matches(
        &self,
        pragma: &CommentedElement<Box<PragmaDirective>>,
        context: &str,
    ) {
        // Check pragma content if specified
        if let Some(expected_content) = &self.content {
            // Extract the pragma identifier (e.g., "solidity" from "pragma solidity ^0.8.0;")
            let actual_content = match pragma.element.as_ref() {
                PragmaDirective::Version(_, ident, _) => &ident.name,
                PragmaDirective::Identifier(_, first_ident, _) => {
                    if let Some(first) = first_ident {
                        &first.name
                    } else {
                        ""
                    }
                }
                PragmaDirective::StringLiteral(_, ident, _) => &ident.name,
            };

            assert_eq!(
                expected_content, actual_content,
                "{}: pragma identifier mismatch",
                context
            );
        }

        // Check second identifier if specified (for Identifier pragmas with two identifiers)
        if let Some(expected_second) = &self.second_identifier {
            if let PragmaDirective::Identifier(_, _, second_ident) = pragma.element.as_ref() {
                if let Some(second) = second_ident {
                    assert_eq!(
                        expected_second, &second.name,
                        "{}: pragma second identifier mismatch",
                        context
                    );
                } else {
                    panic!(
                        "{}: expected pragma with second identifier, but found none",
                        context
                    );
                }
            } else {
                panic!(
                    "{}: expected pragma with second identifier, but found different pragma type",
                    context
                );
            }
        }

        // Check pragma version if specified
        if let Some(expected_version) = &self.version {
            if let PragmaDirective::Version(_, _, version_reqs) = pragma.element.as_ref() {
                // Format the version requirements similar to how it's done in toplevel.rs
                let actual_version = version_reqs
                    .iter()
                    .map(|req| format_version_requirement(req))
                    .collect::<Vec<_>>()
                    .join(" ");

                assert_eq!(
                    expected_version, &actual_version,
                    "{}: pragma version mismatch",
                    context
                );
            } else {
                panic!(
                    "{}: expected pragma with version, but found different pragma type",
                    context
                );
            }
        }

        // Check pragma string value if specified (for StringLiteral pragmas)
        if let Some(expected_string) = &self.string_value {
            if let PragmaDirective::StringLiteral(_, _, string_lit) = pragma.element.as_ref() {
                assert_eq!(
                    expected_string, &string_lit.string,
                    "{}: pragma string value mismatch",
                    context
                );
            } else {
                panic!(
                    "{}: expected pragma with string literal, but found different pragma type",
                    context
                );
            }
        }

        assert_comments(
            &pragma.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &pragma.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for PragmaExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

/// Expectation builder for contract definitions
pub struct ContractExpectation {
    name: String,
    contract_type: Option<String>, // "contract", "interface", or "library"
    base_contracts: Vec<String>,   // inheritance list
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    variables: Vec<VariableExpectation>,
    functions: Vec<FunctionExpectation>,
    structs: Vec<StructExpectation>,
    enums: Vec<EnumExpectation>,
    errors: Vec<ErrorExpectation>,
    events: Vec<EventExpectation>,
    types: Vec<TypeExpectation>,
    using_directives: Vec<UsingExpectation>,
    standalone_comments: Vec<String>,
}

impl ContractExpectation {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            contract_type: None,
            base_contracts: vec![],
            leading_comments: vec![],
            trailing_comments: vec![],
            variables: vec![],
            functions: vec![],
            structs: vec![],
            enums: vec![],
            errors: vec![],
            events: vec![],
            types: vec![],
            using_directives: vec![],
            standalone_comments: vec![],
        }
    }

    pub fn contract_type(mut self, ty: &str) -> Self {
        self.contract_type = Some(ty.to_string());
        self
    }

    pub fn inherits(mut self, base: &str) -> Self {
        self.base_contracts.push(base.to_string());
        self
    }

    pub fn inherits_many(mut self, bases: Vec<&str>) -> Self {
        self.base_contracts
            .extend(bases.iter().map(|s| s.to_string()));
        self
    }

    pub fn variable<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(VariableExpectation) -> VariableExpectation,
    {
        self.variables.push(f(VariableExpectation::new(name)));
        self
    }

    pub fn function<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(FunctionExpectation) -> FunctionExpectation,
    {
        self.functions.push(f(FunctionExpectation::new(name)));
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

    pub fn error<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(ErrorExpectation) -> ErrorExpectation,
    {
        self.errors.push(f(ErrorExpectation::new(name)));
        self
    }

    pub fn event<F>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(EventExpectation) -> EventExpectation,
    {
        self.events.push(f(EventExpectation::new(name)));
        self
    }

    pub fn using_directive<F>(mut self, f: F) -> Self
    where
        F: FnOnce(UsingExpectation) -> UsingExpectation,
    {
        self.using_directives.push(f(UsingExpectation::new()));
        self
    }

    pub fn standalone_comment(mut self, comment: &str) -> Self {
        self.standalone_comments.push(comment.to_string());
        self
    }

    pub(crate) fn assert_matches(&self, contract: &CollectedContract, context: &str) {
        // Check contract name
        let contract_name = match &contract.definition.element.name {
            Some(name) => &name.name,
            None => panic!("{}: contract has no name", context),
        };
        assert_eq!(
            contract_name, &self.name,
            "{}: contract name mismatch",
            context
        );

        // Check contract type if specified
        if let Some(expected_type) = &self.contract_type {
            let actual_type = match &contract.definition.element.ty {
                ContractTy::Contract(_) => "contract",
                ContractTy::Interface(_) => "interface",
                ContractTy::Library(_) => "library",
                ContractTy::Abstract(_) => "abstract",
            };
            assert_eq!(
                actual_type, expected_type,
                "{}: contract type mismatch - expected '{}', found '{}'",
                context, expected_type, actual_type
            );
        }

        // Check inheritance if specified
        if !self.base_contracts.is_empty() {
            let actual_bases: Vec<String> = contract
                .definition
                .element
                .base
                .iter()
                .map(|base| {
                    base.name
                        .identifiers
                        .iter()
                        .map(|id| id.name.clone())
                        .collect::<Vec<_>>()
                        .join(".")
                })
                .collect();

            assert_eq!(
                self.base_contracts, actual_bases,
                "{}: inheritance mismatch - expected {:?}, found {:?}",
                context, self.base_contracts, actual_bases
            );
        }

        // Check comments
        assert_comments(
            &contract.definition.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &contract.definition.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check nested elements
        let contents = &contract.contents;

        assert_eq!(
            self.variables.len(),
            contents.variables.len(),
            "{}: expected {} variables, found {}",
            context,
            self.variables.len(),
            contents.variables.len()
        );
        assert_eq!(
            self.functions.len(),
            contents.functions.len(),
            "{}: expected {} functions, found {}",
            context,
            self.functions.len(),
            contents.functions.len()
        );
        assert_eq!(
            self.structs.len(),
            contents.structs.len(),
            "{}: expected {} structs, found {}",
            context,
            self.structs.len(),
            contents.structs.len()
        );
        assert_eq!(
            self.enums.len(),
            contents.enums.len(),
            "{}: expected {} enums, found {}",
            context,
            self.enums.len(),
            contents.enums.len()
        );
        assert_eq!(
            self.errors.len(),
            contents.errors.len(),
            "{}: expected {} errors, found {}",
            context,
            self.errors.len(),
            contents.errors.len()
        );
        assert_eq!(
            self.events.len(),
            contents.events.len(),
            "{}: expected {} events, found {}",
            context,
            self.events.len(),
            contents.events.len()
        );

        // Check each nested element
        for (i, var_exp) in self.variables.iter().enumerate() {
            var_exp.assert_matches(
                &contents.variables[i],
                &format!("{}.variable[{}]", context, i),
            );
        }

        for (i, func_exp) in self.functions.iter().enumerate() {
            func_exp.assert_matches(
                &contents.functions[i],
                &format!("{}.function[{}]", context, i),
            );
        }

        for (i, struct_exp) in self.structs.iter().enumerate() {
            struct_exp.assert_matches(&contents.structs[i], &format!("{}.struct[{}]", context, i));
        }

        for (i, enum_exp) in self.enums.iter().enumerate() {
            enum_exp.assert_matches(&contents.enums[i], &format!("{}.enum[{}]", context, i));
        }

        for (i, error_exp) in self.errors.iter().enumerate() {
            error_exp.assert_matches(&contents.errors[i], &format!("{}.error[{}]", context, i));
        }

        for (i, event_exp) in self.events.iter().enumerate() {
            event_exp.assert_matches(&contents.events[i], &format!("{}.event[{}]", context, i));
        }

        // Check standalone comments
        assert_eq!(
            self.standalone_comments.len(),
            contents.standalone_comments.len(),
            "{}: expected {} standalone comments, found {}",
            context,
            self.standalone_comments.len(),
            contents.standalone_comments.len()
        );

        for (i, expected_comment) in self.standalone_comments.iter().enumerate() {
            assert_comment_text(
                &contents.standalone_comments[i],
                expected_comment,
                &format!("{}.standalone[{}]", context, i),
            );
        }
    }
}

impl CommentExpectation for ContractExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

/// Expectation builder for import statements
pub struct ImportExpectation {
    path: Option<String>,
    import_type: Option<String>,            // "plain", "global", "rename"
    global_alias: Option<String>,           // For GlobalSymbol variant
    symbols: Vec<(String, Option<String>)>, // For Rename variant: (original, alias)
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl ImportExpectation {
    pub(crate) fn new() -> Self {
        Self {
            path: None,
            import_type: None,
            global_alias: None,
            symbols: vec![],
            leading_comments: vec![],
            trailing_comments: vec![],
        }
    }

    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn plain(mut self) -> Self {
        self.import_type = Some("plain".to_string());
        self
    }

    pub fn global_as(mut self, alias: &str) -> Self {
        self.import_type = Some("global".to_string());
        self.global_alias = Some(alias.to_string());
        self
    }

    pub fn symbol(mut self, original: &str, alias: Option<&str>) -> Self {
        if self.import_type.is_none() {
            self.import_type = Some("rename".to_string());
        }
        self.symbols
            .push((original.to_string(), alias.map(|s| s.to_string())));
        self
    }

    pub(crate) fn assert_matches(&self, import: &CommentedElement<Box<Import>>, context: &str) {
        // Check import type and details
        match import.element.as_ref() {
            Import::Plain(import_path, _) => {
                if let Some(expected_type) = &self.import_type {
                    assert_eq!(
                        expected_type, "plain",
                        "{}: expected plain import but got different type",
                        context
                    );
                }

                if let Some(expected_path) = &self.path {
                    let actual_path = extract_import_path(&import_path);
                    assert_eq!(
                        expected_path, &actual_path,
                        "{}: import path mismatch",
                        context
                    );
                }
            }
            Import::GlobalSymbol(import_path, identifier, _) => {
                if let Some(expected_type) = &self.import_type {
                    assert_eq!(
                        expected_type, "global",
                        "{}: expected global import but got different type",
                        context
                    );
                }

                if let Some(expected_path) = &self.path {
                    let actual_path = extract_import_path(&import_path);
                    assert_eq!(
                        expected_path, &actual_path,
                        "{}: import path mismatch",
                        context
                    );
                }

                if let Some(expected_alias) = &self.global_alias {
                    assert_eq!(
                        expected_alias, &identifier.name,
                        "{}: global import alias mismatch",
                        context
                    );
                }
            }
            Import::Rename(import_path, symbols, _) => {
                if let Some(expected_type) = &self.import_type {
                    assert_eq!(
                        expected_type, "rename",
                        "{}: expected rename import but got different type",
                        context
                    );
                }

                if let Some(expected_path) = &self.path {
                    let actual_path = extract_import_path(&import_path);
                    assert_eq!(
                        expected_path, &actual_path,
                        "{}: import path mismatch",
                        context
                    );
                }

                // Check symbols
                if !self.symbols.is_empty() {
                    assert_eq!(
                        self.symbols.len(),
                        symbols.len(),
                        "{}: import symbol count mismatch",
                        context
                    );

                    for (i, (expected_orig, expected_alias)) in self.symbols.iter().enumerate() {
                        let (actual_orig, actual_alias) = &symbols[i];
                        assert_eq!(
                            expected_orig, &actual_orig.name,
                            "{}: symbol {} name mismatch",
                            context, i
                        );

                        match (expected_alias, actual_alias) {
                            (Some(exp), Some(act)) => {
                                assert_eq!(
                                    exp, &act.name,
                                    "{}: symbol {} alias mismatch",
                                    context, i
                                );
                            }
                            (None, None) => {}
                            _ => panic!("{}: symbol {} alias presence mismatch", context, i),
                        }
                    }
                }
            }
        }

        // Check comments
        assert_comments(
            &import.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &import.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for ImportExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

/// Expectation builder for using directives
pub struct UsingExpectation {
    library: Option<String>, // The library being used (e.g., "SafeMath", "StringUtils")
    target_type: Option<String>, // The type it's used for (e.g., "uint256", "*" for all)
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
}

impl UsingExpectation {
    pub(crate) fn new() -> Self {
        Self {
            library: None,
            target_type: None,
            leading_comments: vec![],
            trailing_comments: vec![],
        }
    }

    pub fn library(mut self, lib: &str) -> Self {
        self.library = Some(lib.to_string());
        self
    }

    pub fn for_type(mut self, ty: &str) -> Self {
        self.target_type = Some(ty.to_string());
        self
    }

    pub(crate) fn assert_matches(&self, using: &CommentedElement<Box<Using>>, context: &str) {
        // Check library if specified
        if let Some(expected_lib) = &self.library {
            let actual_lib = match &using.element.list {
                UsingList::Library(path) => {
                    // Extract library name from identifier path
                    path.identifiers
                        .iter()
                        .map(|id| id.name.clone())
                        .collect::<Vec<_>>()
                        .join(".")
                }
                _ => panic!(
                    "{}: expected library using directive but found something else",
                    context
                ),
            };
            assert_eq!(
                expected_lib, &actual_lib,
                "{}: library name mismatch - expected '{}', found '{}'",
                context, expected_lib, actual_lib
            );
        }

        // Check target type if specified
        if let Some(expected_type) = &self.target_type {
            let actual_type = if let Some(ty) = &using.element.ty {
                extract_type_string(ty)
            } else {
                "*".to_string() // None means "for *" (all types)
            };
            assert_eq!(
                expected_type, &actual_type,
                "{}: target type mismatch - expected '{}', found '{}'",
                context, expected_type, actual_type
            );
        }

        assert_comments(
            &using.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &using.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );
    }
}

impl CommentExpectation for UsingExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}
