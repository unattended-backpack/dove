//! Data structures for the two-pass formatting system.
//!
//! This module defines the intermediate representation (IR) that flows between
//! the collection pass and the formatting pass. These structures represent the
//! "decorated AST" with associated comments and parsed metadata.

use solang_parser::pt::*;
use std::collections::HashMap;

/// The main data structure returned by the collection pass.
/// Contains all source unit elements organized by type with associated comments.
#[derive(Default, Debug, Clone)]
pub struct CollectedElements {
    /// Pragma directives (solidity version, experimental features, etc.)
    pub pragmas: Vec<CommentedElement<Box<PragmaDirective>>>,
    /// Import statements organized by source
    pub imports: Vec<CommentedElement<Box<Import>>>,
    /// Using directives for library extensions
    pub using_directives: Vec<CommentedElement<Box<Using>>>,
    /// Top-level error definitions
    pub errors: Vec<CollectedError>,
    /// Type alias definitions
    pub types: Vec<CommentedElement<Box<TypeDefinition>>>,
    /// Top-level enum definitions
    pub enums: Vec<CollectedEnum>,
    /// Top-level struct definitions
    pub structs: Vec<CollectedStruct>,
    /// Top-level variable definitions
    pub variables: Vec<CommentedElement<Box<VariableDefinition>>>,
    /// Top-level function definitions
    pub functions: Vec<CollectedFunction>,
    /// Contract definitions with their contents
    pub contracts: Vec<CollectedContract>,
    /// Comments not associated with any specific AST element
    pub standalone_comments: Vec<Comment>,
}

impl CollectedElements {
    /// Creates a new empty `CollectedElements` instance.
    pub fn new() -> Self {
        Self {
            pragmas: Vec::new(),
            imports: Vec::new(),
            contracts: Vec::new(),
            enums: Vec::new(),
            structs: Vec::new(),
            functions: Vec::new(),
            errors: Vec::new(),
            variables: Vec::new(),
            types: Vec::new(),
            using_directives: Vec::new(),
            standalone_comments: Vec::new(),
        }
    }
}

/// A generic wrapper that associates an AST element with its comments.
///
/// This structure pairs any AST element with its associated leading and trailing comments,
/// allowing the formatter to preserve comment placement during code transformation.
#[derive(Debug, Clone)]
pub struct CommentedElement<T> {
    /// The actual AST element (pragma, function, contract, etc.)
    pub element: T,
    /// Comments that appear before this element
    pub leading_comments: Vec<Comment>,
    /// Comments that appear after this element on the same line
    pub trailing_comments: Vec<Comment>,
    /// Comments that appear within type expressions (for elements with types)
    pub type_comments: Option<Box<TypeExpressionComments>>,
}

/// A catch clause with associated comments.
///
/// Represents a single catch clause in a try-catch statement with its comments.
#[derive(Debug, Clone)]
pub struct CommentedCatchClause {
    /// The catch clause itself (Simple or Named)
    pub clause: CatchClause,
    /// Comments that appear before this catch clause
    pub leading_comments: Vec<Comment>,
    /// Comments that appear after the catch clause header on the same line
    pub trailing_comments: Vec<Comment>,
    /// The statements within the catch block
    pub body_statements: Vec<CommentedStatement>,
    /// Standalone comments within the catch block
    pub body_standalone_comments: Vec<Comment>,
}

/// An if-else branch with associated comments.
///
/// Represents the else branch of an if statement with its comments and body.
#[derive(Debug, Clone)]
pub struct CommentedElseBranch {
    /// Comments that appear before the else keyword
    pub leading_comments: Vec<Comment>,
    /// Comments that appear after else on the same line
    pub trailing_comments: Vec<Comment>,
    /// The else statement (could be a block or another if)
    pub statement: Statement,
    /// For block else statements, the nested statements
    pub nested_statements: Option<Vec<CommentedStatement>>,
    /// For block else statements, standalone comments
    pub nested_standalone_comments: Option<Vec<Comment>>,
}

/// Comments associated with for loop components (init, condition, update).
///
/// Represents comments that appear within the for(...) header.
#[derive(Debug, Clone)]
pub struct ForLoopComponentComments {
    /// Comments before the init statement
    pub init_leading: Vec<Comment>,
    /// Comments after the init statement (before semicolon)
    pub init_trailing: Vec<Comment>,
    /// Comments before the condition expression
    pub condition_leading: Vec<Comment>,
    /// Comments after the condition expression (before semicolon)
    pub condition_trailing: Vec<Comment>,
    /// Comments before the update expression
    pub update_leading: Vec<Comment>,
    /// Comments after the update expression (before closing paren)
    pub update_trailing: Vec<Comment>,
}

/// Comments associated with function parameters in the signature.
///
/// Represents comments for individual parameters within the function declaration.
#[derive(Debug, Clone)]
pub struct FunctionParameterComments {
    /// Comments that appear before this parameter
    pub leading: Vec<Comment>,
    /// Comments that appear after this parameter
    pub trailing: Vec<Comment>,
    /// Comments that appear within the parameter's type expression
    pub type_comments: Option<Box<TypeExpressionComments>>,
}

/// Comments associated with function signature components (parameters and returns).
///
/// Captures comments that appear within the function signature, including
/// comments for individual parameters and return values.
#[derive(Debug, Clone)]
pub struct FunctionSignatureComments {
    /// Comments for each parameter, indexed by parameter position
    pub parameters: Vec<FunctionParameterComments>,
    /// Comments for each return value, indexed by return position
    pub returns: Vec<FunctionParameterComments>,
}

/// A statement with associated comments.
///
/// Similar to `CommentedElement` but specifically for Solidity statements within function bodies.
#[derive(Debug, Clone)]
pub struct CommentedStatement {
    /// The Solidity statement (if, while, assignment, etc.)
    pub statement: Statement,
    /// Comments that appear before this statement
    pub leading_comments: Vec<Comment>,
    /// Comments that appear after this statement on the same line
    pub trailing_comments: Vec<Comment>,
    /// For block statements, contains the nested commented statements
    pub nested_statements: Option<Vec<CommentedStatement>>,
    /// For block statements, contains standalone comments not associated with any nested statement
    pub nested_standalone_comments: Option<Vec<Comment>>,
    /// For assembly blocks, contains the collected YUL block with properly associated comments
    pub yul_block: Option<CollectedYulBlock>,
    /// For try statements, contains all catch clauses with their associated comments
    pub catch_clauses: Option<Vec<CommentedCatchClause>>,
    /// For if statements, contains the else branch with its comments (if present)
    pub else_branch: Option<Box<CommentedElseBranch>>,
    /// For for loops, contains comments associated with the loop components (init, condition, update)
    pub for_component_comments: Option<Box<ForLoopComponentComments>>,
}

/// A contract with its definition and organized internal elements.
///
/// Represents a complete contract including its declaration (name, inheritance, etc.)
/// and all internal elements (functions, variables, events, etc.) organized by type.
#[derive(Debug, Clone)]
pub struct CollectedContract {
    /// The contract declaration with its associated comments
    pub definition: CommentedElement<Box<ContractDefinition>>,
    /// All elements contained within this contract, organized by type
    pub contents: CollectedContractElements,
}

/// A function with its definition and internal comments.
///
/// Captures function definition and any comments within the function body.
#[derive(Debug, Clone)]
pub struct CollectedFunction {
    /// The function declaration with its associated comments
    pub definition: CommentedElement<Box<FunctionDefinition>>,
    /// Statements within the function body with their associated comments
    pub body_statements: Vec<CommentedStatement>,
    /// Comments within the function body not associated with any statement
    pub body_standalone_comments: Vec<Comment>,
    /// Comments associated with function parameters and return values in the signature
    pub signature_comments: Option<Box<FunctionSignatureComments>>,
}

/// A struct with its definition and field-level comments.
///
/// Captures struct definition and comments for individual fields.
#[derive(Debug, Clone)]
pub struct CollectedStruct {
    /// The struct declaration with its associated comments
    pub definition: CommentedElement<Box<StructDefinition>>,
    /// Individual struct fields with their associated comments
    pub fields: Vec<CommentedElement<Box<VariableDeclaration>>>,
}

/// An enum with its definition and value-level comments.
///
/// Captures enum definition and comments for individual enum values.
#[derive(Debug, Clone)]
pub struct CollectedEnum {
    /// The enum declaration with its associated comments
    pub definition: CommentedElement<Box<EnumDefinition>>,
    /// Individual enum values with their associated comments
    pub values: Vec<CommentedElement<Option<Identifier>>>,
}

/// An event with its definition and parameter-level comments.
///
/// Captures event definition and comments for individual parameters.
#[derive(Debug, Clone)]
pub struct CollectedEvent {
    /// The event declaration with its associated comments
    pub definition: CommentedElement<Box<EventDefinition>>,
    /// Individual event parameters with their associated comments
    pub parameters: Vec<CommentedElement<Box<EventParameter>>>,
}

/// An error with its definition and parameter-level comments.
///
/// Captures error definition and comments for individual parameters.
#[derive(Debug, Clone)]
pub struct CollectedError {
    /// The error declaration with its associated comments
    pub definition: CommentedElement<Box<ErrorDefinition>>,
    /// Individual error parameters with their associated comments
    pub parameters: Vec<CommentedElement<Box<ErrorParameter>>>,
}

/// Container for all elements within a contract, organized by type.
///
/// This structure separates contract members into distinct collections, enabling
/// the formatter to apply different ordering and spacing rules to each element type.
#[derive(Default, Debug, Clone)]
pub struct CollectedContractElements {
    /// Custom struct definitions within the contract
    pub structs: Vec<CollectedStruct>,
    /// Event definitions for contract logging
    pub events: Vec<CollectedEvent>,
    /// Enum definitions for named constants
    pub enums: Vec<CollectedEnum>,
    /// Custom error definitions
    pub errors: Vec<CollectedError>,
    /// State variable declarations
    pub variables: Vec<CommentedElement<Box<VariableDefinition>>>,
    /// Function definitions (constructors, external, internal, etc.)
    pub functions: Vec<CollectedFunction>,
    /// Type alias definitions (using type Foo is Bar)
    pub types: Vec<CommentedElement<Box<TypeDefinition>>>,
    /// Using directives (using Lib for Type)
    pub using_directives: Vec<CommentedElement<Box<Using>>>,
    /// Comments not associated with any specific contract element
    pub standalone_comments: Vec<Comment>,
}

impl CollectedContractElements {
    /// Creates a new empty `CollectedContractElements` instance.
    pub fn new() -> Self {
        Self {
            structs: Vec::new(),
            events: Vec::new(),
            enums: Vec::new(),
            errors: Vec::new(),
            variables: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
            using_directives: Vec::new(),
            standalone_comments: Vec::new(),
        }
    }
}

// Comment parsing data structures - define the shape of parsed comment metadata

/// Parsed contract-level comment information.
///
/// Extracts structured metadata from contract documentation comments,
/// including NatSpec tags and custom project-specific annotations.
#[derive(Debug, Clone)]
pub struct ParsedContractComment {
    /// Contract title from @title tag
    pub title: Option<String>,
    /// Author information from @author tag  
    pub author: Option<String>,
    /// Custom project annotation from @custom:terry tag
    pub custom_terry: Option<String>,
    /// Custom date annotation from @custom:date tag
    pub custom_date: Option<String>,
    /// Other unrecognized tags for extensibility
    pub other_tags: Vec<String>,
    /// Main description text of the contract
    pub description: String,
}

/// Parsed struct-level comment information.
///
/// Captures documentation for struct definitions including field-level documentation.
#[derive(Debug, Clone)]
pub struct ParsedStructComment {
    /// Main description of the struct's purpose  
    pub description: String,
    /// Documentation for individual struct fields (field_name -> description)
    pub field_docs: HashMap<String, String>,
}

/// Field information for struct documentation.
///
/// Represents metadata about a struct field for documentation generation.
#[derive(Debug, Clone)]
pub struct StructField {
    /// The field name as it appears in the struct
    pub name: String,
    /// The Solidity type of this field (formatted as string)
    pub type_name: String,
}

/// Function parameter information.
///
/// Represents metadata about a function parameter for documentation generation.
#[derive(Debug, Clone)]
pub struct FunctionParam {
    /// The parameter name as declared in the function signature
    pub name: String,
    /// The Solidity type of this parameter (formatted as string)
    pub type_name: String,
}

/// Function return value information.
///
/// Represents metadata about a function's return value for documentation generation.
#[derive(Debug, Clone)]
pub struct FunctionReturn {
    /// The Solidity type of the return value (formatted as string)
    pub type_name: String,
}

/// Parsed function-level comment information.
///
/// Extracts NatSpec documentation from function comments including parameter
/// and return value documentation.
#[derive(Debug, Clone)]
pub struct ParsedFunctionComment {
    /// Main description of the function's behavior
    pub description: String,
    /// Documentation for individual parameters (param_name -> description)
    pub param_docs: HashMap<String, String>,
    /// Documentation for the return value, if any
    pub return_doc: Option<String>,
}

/// Parsed state variable comment information.
///
/// Captures documentation for contract state variables.
#[derive(Debug, Clone)]
pub struct ParsedStateVariableComment {
    /// Main description of the state variable's purpose
    pub description: String,
    /// Other tags found in the comment for extensibility
    pub other_tags: Vec<String>,
}

/// Mapping type information for documentation.
///
/// Represents metadata about Solidity mapping types for documentation purposes.
#[derive(Debug, Clone)]
pub struct MappingInfo {
    /// The key type of the mapping (e.g., "address", "uint256")
    pub key_type: String,
    /// The value type of the mapping (e.g., "uint256", "bool")
    pub value_type: String,
    /// Nesting level for nested mappings (0 = simple mapping)
    pub level: usize,
}

/// Grouped import organization.
///
/// Organizes import statements into logical groups for consistent formatting.
/// The formatter applies different spacing and ordering rules to each group.
#[derive(Debug, Clone)]
pub struct GroupedImports {
    /// External package imports (paths starting with "@")
    pub external: Vec<Import>,
    /// Interface imports (paths starting with "interfaces")
    pub interfaces: Vec<Import>,
    /// Local project imports (all other import paths)
    pub local: Vec<Import>,
}

/// Grouped imports with comments.
///
/// Like `GroupedImports` but each import is wrapped with its associated comments.
/// This is the comment-aware version used during the formatting phase.
#[derive(Debug, Clone)]
pub struct GroupedCommentedImports {
    /// External package imports with their comments (paths starting with "@")
    pub external: Vec<CommentedElement<Box<Import>>>,
    /// Interface imports with their comments (paths starting with "interfaces")
    pub interfaces: Vec<CommentedElement<Box<Import>>>,
    /// Local project imports with their comments (all other import paths)
    pub local: Vec<CommentedElement<Box<Import>>>,
}

/// Parsed enum-level comment information.
///
/// Captures documentation for enum definitions including value-level documentation.
#[derive(Debug, Clone)]
pub struct ParsedEnumComment {
    /// Main description of the enum's purpose
    pub description: String,
    /// Documentation for individual enum values (value_name -> description)
    pub value_docs: HashMap<String, String>,
}

/// Parsed event-level comment information.
///
/// Captures documentation for event definitions including parameter documentation.
#[derive(Debug, Clone)]
pub struct ParsedEventComment {
    /// Main description of the event's purpose and when it's emitted
    pub description: String,
    /// Documentation for individual event parameters (param_name -> description)
    pub param_docs: HashMap<String, String>,
}

/// Parsed error-level comment information.
///
/// Captures documentation for custom error definitions including parameter documentation.
#[derive(Debug, Clone)]
pub struct ParsedErrorComment {
    /// Main description of the error condition and when it occurs
    pub description: String,
    /// Documentation for individual error parameters (param_name -> description)
    pub param_docs: HashMap<String, String>,
}

/// A YUL statement with associated comments.
///
/// Similar to `CommentedStatement` but specifically for YUL statements within assembly blocks.
/// This allows proper comment association within YUL code.
#[derive(Debug, Clone)]
pub struct CommentedYulStatement {
    /// The YUL statement (assignment, variable declaration, if, for, switch, etc.)
    pub statement: YulStatement,
    /// Comments that appear before this YUL statement
    pub leading_comments: Vec<Comment>,
    /// Comments that appear after this YUL statement on the same line
    pub trailing_comments: Vec<Comment>,
    /// For block-like YUL statements (if, for), contains nested YUL statements
    pub nested_statements: Option<Vec<CommentedYulStatement>>,
    /// For block-like YUL statements, contains standalone comments not associated with any nested statement
    pub nested_standalone_comments: Option<Vec<Comment>>,
    /// For switch statements, contains the commented cases
    pub switch_cases: Option<Vec<CommentedYulCase>>,
}

/// A YUL block with collected statements and comments.
///
/// Represents the contents of an assembly block with properly associated comments.
#[derive(Debug, Clone)]
pub struct CollectedYulBlock {
    /// The YUL statements within the block, each with their associated comments
    pub statements: Vec<CommentedYulStatement>,
    /// Standalone comments within the YUL block not associated with any statement
    pub standalone_comments: Vec<Comment>,
}

/// A YUL switch case with associated comments.
///
/// Represents a single case or default clause within a YUL switch statement,
/// including its pattern, block, and associated comments.
#[derive(Debug, Clone)]
pub struct CommentedYulCase {
    /// The switch case (either Case or Default variant)
    pub case: YulSwitchOptions,
    /// Comments that appear before this case keyword
    pub leading_comments: Vec<Comment>,
    /// Comments that appear after this case on the same line
    pub trailing_comments: Vec<Comment>,
    /// The statements within the case block, each with their associated comments
    pub body_statements: Vec<CommentedYulStatement>,
    /// Standalone comments within the case block not associated with any statement
    pub body_standalone_comments: Vec<Comment>,
}

/// Comments associated with type expressions.
///
/// Captures comments that appear within type expressions like mappings, arrays, and function types.
/// This is particularly useful for complex type expressions where developers want to document
/// the meaning of keys and values in mappings or parameters in function types.
#[derive(Debug, Clone, Default)]
pub struct TypeExpressionComments {
    /// For mapping types: comments before the key type
    pub key_leading_comments: Vec<Comment>,
    /// For mapping types: comments after the key type (before =>)
    pub key_trailing_comments: Vec<Comment>,
    /// For mapping types: comments before the value type
    pub value_leading_comments: Vec<Comment>,
    /// For mapping types: comments after the value type
    pub value_trailing_comments: Vec<Comment>,
    /// For nested mappings: comments for the nested value type expression
    pub nested_type_comments: Option<Box<TypeExpressionComments>>,
    /// For array types: comments before the array element type
    pub element_leading_comments: Vec<Comment>,
    /// For array types: comments after the array element type
    pub element_trailing_comments: Vec<Comment>,
    /// For function types: comments for each parameter
    pub function_param_comments: Vec<FunctionParameterComments>,
    /// For function types: comments for return types
    pub function_return_comments: Vec<FunctionParameterComments>,
}

impl TypeExpressionComments {
    /// Creates a new empty `TypeExpressionComments` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if this structure contains any comments.
    pub fn has_comments(&self) -> bool {
        !self.key_leading_comments.is_empty()
            || !self.key_trailing_comments.is_empty()
            || !self.value_leading_comments.is_empty()
            || !self.value_trailing_comments.is_empty()
            || self
                .nested_type_comments
                .as_ref()
                .map_or(false, |n| n.has_comments())
            || !self.element_leading_comments.is_empty()
            || !self.element_trailing_comments.is_empty()
            || !self.function_param_comments.is_empty()
            || !self.function_return_comments.is_empty()
    }
}
