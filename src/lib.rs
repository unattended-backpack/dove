#![deny(missing_docs)]

//! Dove - Solidity formatter library
//!
//! This library provides functionality for formatting Solidity source code
//! using an intermediate representation (IR) based approach with Philip Wadler's
//! "A prettier printer" algorithm.
//!
//! # Quick Start
//!
//! ```rust
//! use dove::format_solidity;
//!
//! let solidity_code = r#"
//!     contract MyContract {
//!         function test() public {}
//!     }
//! "#;
//!
//! let formatted = format_solidity(solidity_code).unwrap();
//! println!("{}", formatted);
//! ```
//!
//! # Configuration
//!
//! ```rust,ignore
//! use dove::{format_solidity_with_config, FormatConfig};
//!
//! let config = FormatConfig::builder()
//!     .max_line_width(100)
//!     .indent_size(4)
//!     .build();
//!
//! let formatted = format_solidity_with_config(source, &config).unwrap();
//! ```

use std::fmt;

// Public API modules
pub mod ir_builder;
pub mod collector;
pub mod printer;
pub mod peck;

use crate::printer::Printer;

/// Configuration options for Solidity formatting.
///
/// This struct allows customization of various formatting behaviors including
/// line width limits, indentation, and other style preferences.
#[derive(Debug, Clone, PartialEq)]
pub struct FormatConfig {
    /// Maximum line width before wrapping (default: 80)
    pub max_line_width: usize,
    /// Number of spaces per indentation level (default: 2)
    pub indent_size: usize,
}

impl Default for FormatConfig {
    fn default() -> Self {
        FormatConfig {
            max_line_width: 80,
            indent_size: 2,
        }
    }
}

impl AsRef<FormatConfig> for FormatConfig {
    fn as_ref(&self) -> &FormatConfig {
        self
    }
}

/// Builder for constructing [`FormatConfig`] instances.
///
/// Uses the builder pattern to provide a fluent interface for setting
/// configuration options with sensible defaults.
#[derive(Debug, Default)]
pub struct FormatConfigBuilder {
    max_line_width: Option<usize>,
    indent_size: Option<usize>,
}

impl FormatConfigBuilder {
    /// Set the maximum line width before wrapping.
    ///
    /// # Arguments
    ///
    /// * `width` - Maximum characters per line
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dove::FormatConfig;
    ///
    /// let config = FormatConfig::builder()
    ///     .max_line_width(120)
    ///     .build();
    /// ```
    pub fn max_line_width(mut self, width: usize) -> Self {
        self.max_line_width = Some(width);
        self
    }

    /// Set the number of spaces per indentation level.
    ///
    /// # Arguments
    ///
    /// * `size` - Spaces per indent
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dove::FormatConfig;
    ///
    /// let config = FormatConfig::builder()
    ///     .indent_size(2)
    ///     .build();
    /// ```
    pub fn indent_size(mut self, size: usize) -> Self {
        self.indent_size = Some(size);
        self
    }

    /// Build the final [`FormatConfig`] with specified options and defaults.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dove::FormatConfig;
    ///
    /// let config = FormatConfig::builder()
    ///     .max_line_width(100)
    ///     .build();
    /// ```
    pub fn build(self) -> FormatConfig {
        let defaults = FormatConfig::default();
        FormatConfig {
            max_line_width: self.max_line_width.unwrap_or(defaults.max_line_width),
            indent_size: self.indent_size.unwrap_or(defaults.indent_size),
        }
    }
}

impl FormatConfig {
    /// Create a new builder for configuring formatting options.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use dove::FormatConfig;
    ///
    /// let config = FormatConfig::builder()
    ///     .max_line_width(100)
    ///     .indent_size(2)
    ///     .build();
    /// ```
    pub fn builder() -> FormatConfigBuilder {
        FormatConfigBuilder::default()
    }
}

/// Errors that can occur during Solidity formatting.
#[derive(Debug)]
pub enum FormatError {
    /// Parsing error occurred while processing Solidity source code.
    ParseError {
        /// Specific type of parsing error
        kind: ParseErrorKind,
        /// Human-readable error message
        message: String,
    },
    /// I/O error occurred during file operations.
    IoError(std::io::Error),
    /// Invalid configuration provided.
    InvalidConfig {
        /// Name of the invalid configuration field
        field: String,
        /// Description of why the configuration is invalid
        message: String,
    },
}

/// Specific types of parsing errors.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// Encountered an unexpected token during parsing
    UnexpectedToken,
    /// Invalid syntax structure found
    InvalidSyntax,
    /// Unsupported Solidity language feature
    UnsupportedFeature,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::ParseError { kind, message } => {
                write!(f, "Parse error ({:?}): {}", kind, message)?;
                Ok(())
            }
            FormatError::IoError(err) => write!(f, "I/O error: {}", err),
            FormatError::InvalidConfig { field, message } => {
                write!(f, "Invalid configuration for '{}': {}", field, message)
            }
        }
    }
}

impl std::error::Error for FormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FormatError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FormatError {
    fn from(err: std::io::Error) -> Self {
        FormatError::IoError(err)
    }
}

/// Format Solidity source code with default settings.
///
/// This is the simplest way to format Solidity code, using sensible defaults
/// for all formatting options (80 character line width, 4-space indentation).
///
/// # Arguments
///
/// * `source` - Solidity source code to format
///
/// # Returns
///
/// Formatted Solidity code or an error if parsing/formatting fails.
///
/// # Errors
///
/// Returns [`FormatError::ParseError`] if the source code contains syntax errors
/// or unsupported language features.
///
/// # Examples
///
/// ```rust
/// use dove::format_solidity;
///
/// let source = "contract Test{function f()public{}}";
/// let formatted = format_solidity(source)?;
///
/// // Output:
/// // contract Test {
/// //     function f() public {}
/// // }
/// # Ok::<(), dove::FormatError>(())
/// ```
pub fn format_solidity<S: AsRef<str>>(source: S) -> Result<String, FormatError> {
    format_solidity_with_config(source, FormatConfig::default())
}

/// Format Solidity source code with custom configuration.
///
/// Provides full control over formatting behavior through a [`FormatConfig`]
/// instance. Use [`FormatConfig::builder()`] for convenient configuration.
///
/// # Arguments
///
/// * `source` - Solidity source code to format
/// * `config` - Configuration options for formatting behavior
///
/// # Returns
///
/// Formatted Solidity code or an error if parsing/formatting fails.
///
/// # Errors
///
/// Returns [`FormatError::ParseError`] if the source code contains syntax errors.
/// Returns [`FormatError::InvalidConfig`] if configuration is invalid.
///
/// # Examples
///
/// ```rust
/// use dove::{format_solidity_with_config, FormatConfig};
///
/// let config = FormatConfig::builder()
///     .max_line_width(100)
///     .indent_size(2)
///     .build();
///
/// let source = "contract Test{function f()public{}}";
/// let formatted = format_solidity_with_config(source, &config)?;
/// # Ok::<(), dove::FormatError>(())
/// ```
pub fn format_solidity_with_config<S: AsRef<str>, C: AsRef<FormatConfig>>(
    source: S,
    config: C,
) -> Result<String, FormatError> {
    let source = source.as_ref();
    let config = config.as_ref();

    // Parse Solidity source code into AST elements and comments.
    let (source_unit, comments) = solang_parser::parse(source, 0).map_err(|errors| {
        let first_error = errors.into_iter().next().unwrap_or_else(|| {
            solang_parser::diagnostics::Diagnostic::parser_error(
                solang_parser::pt::Loc::Builtin,
                "Unknown parsing error".to_string(),
            )
        });

        FormatError::ParseError {
            kind: ParseErrorKind::InvalidSyntax,
            message: format!("{:?}", first_error),
        }
    })?;

    // Collect comments for association with AST elements.
    let collected = collector::collect_source_unit(&source_unit, &comments, source);

    // Construct an IR from the commented AST elements for printing.
    let ir = ir_builder::build_ir(&collected);

    // Print the IR into its properly-formatted output.
    let mut printer = Printer::with_config(config.max_line_width, config.indent_size);
    Ok(printer.print(&ir))
}

/// Extract the public interface from Solidity source code.
///
/// The `peck` command analyzes Solidity source code and extracts the public
/// interface, suitable for generating a Solidity interface file.
///
/// This transformation:
/// - Changes `contract X` to `interface IX`
/// - Transforms `@title X` to `@title X Interface`
/// - Converts `@custom:param` to `@param` and `@custom:return` to `@return`
/// - Filters out events and modifiers
/// - Converts public constants/mappings to getter function signatures
/// - Converts public/external functions to external signatures without bodies
///
/// # Arguments
///
/// * `source` - Solidity source code to extract interface from
///
/// # Returns
///
/// The extracted interface as formatted Solidity code, or an error if parsing fails.
///
/// # Errors
///
/// Returns [`FormatError::ParseError`] if the source code contains syntax errors.
pub fn peck_solidity<S: AsRef<str>>(source: S) -> Result<String, FormatError> {
    peck_solidity_with_config(source, FormatConfig::default())
}

/// Extract the public interface from Solidity source code with custom configuration.
///
/// See [`peck_solidity`] for details on the transformation.
///
/// # Arguments
///
/// * `source` - Solidity source code to extract interface from
/// * `config` - Configuration options for formatting behavior
///
/// # Returns
///
/// The extracted interface as formatted Solidity code, or an error if parsing fails.
///
/// # Errors
///
/// Returns [`FormatError::ParseError`] if the source code contains syntax errors.
pub fn peck_solidity_with_config<S: AsRef<str>, C: AsRef<FormatConfig>>(
    source: S,
    config: C,
) -> Result<String, FormatError> {
    let source = source.as_ref();
    let config = config.as_ref();

    // Parse Solidity source code into AST elements and comments.
    let (source_unit, comments) = solang_parser::parse(source, 0).map_err(|errors| {
        let first_error = errors.into_iter().next().unwrap_or_else(|| {
            solang_parser::diagnostics::Diagnostic::parser_error(
                solang_parser::pt::Loc::Builtin,
                "Unknown parsing error".to_string(),
            )
        });

        FormatError::ParseError {
            kind: ParseErrorKind::InvalidSyntax,
            message: format!("{:?}", first_error),
        }
    })?;

    // Collect comments for association with AST elements.
    let collected = collector::collect_source_unit(&source_unit, &comments, source);

    // Transform to interface representation.
    let interface = peck::transform_to_interface(&collected);

    // Construct an IR from the interface elements for printing.
    let ir = ir_builder::build_ir(&interface);

    // Print the IR into its properly-formatted output.
    let mut printer = Printer::with_config(config.max_line_width, config.indent_size);
    Ok(printer.print(&ir))
}
