//! Enhanced Intermediate Representation for Wadler-style formatting.
//!
//! This module implements the IR elements used by the Solidity formatter,
//! based on Philip Wadler's "A prettier printer" algorithm. The IR allows
//! the formatter to defer layout decisions until the final printing stage,
//! enabling intelligent line-wrapping based on content width and complexity.

/// Intermediate Representation elements for flexible formatting.
///
/// Based on Wadler's prettier algorithm, these elements allow the formatter
/// to build a structured representation of the output with break opportunities,
/// then let the printer decide the final layout based on line width constraints.
#[derive(Debug, Clone, PartialEq)]
pub enum IRElement {
    /// Raw text content to be output as-is.
    Text(String),

    /// Unconditional line break - always produces a newline.
    /// Used for mandatory breaks like after semicolons or between statements.
    HardLineBreak,

    /// Conditional line break - becomes a space if the group fits on one line,
    /// or a newline with appropriate indentation if the group needs to break.
    /// This is the core element that enables intelligent line wrapping.
    SoftLineBreak,

    /// Conditional line break - becomes nothing if the group fits on one line,
    /// or a newline with appropriate indentation if the group needs to break.
    /// Use this when you don't want a space when the group stays on one line.
    SoftestLineBreak,

    /// Conditional line break with continuation indentation - becomes a space if
    /// the content fits on one line, or a newline with additional 2-space indentation
    /// for wrapped continuation lines. Used for NatSpec tag descriptions.
    SoftLineBreakWithContinuation,

    /// A formatting group that should be kept on one line if possible.
    /// If the group's total width exceeds the line limit, all SoftLineBreaks
    /// within the group will be converted to actual line breaks.
    /// This is the key element for "all or nothing" formatting decisions.
    Group(Box<[IRElement]>),

    /// An indentation context - all SoftLineBreaks within this block will
    /// be indented by an additional level if they break to new lines.
    /// This provides proper nesting structure for complex expressions.
    Indent(Box<[IRElement]>),

    /// A comment with content to be formatted by the printer.
    /// The printer decides whether to render as line comment (`// ...`),
    /// wrapped line comments, or block comment (`/* ... */`) based on
    /// line length limits and current indentation.
    Comment {
        /// The comment text content.
        text: String,
        /// Whether this is a doc comment (uses `///` or `/** */`).
        is_doc: bool,
    },
}

impl IRElement {
    /// Create a text element from any string-like input.
    pub fn text(s: impl Into<String>) -> Self {
        IRElement::Text(s.into())
    }

    /// Create a group from a vector of elements.
    pub fn group(elements: Vec<IRElement>) -> Self {
        IRElement::Group(elements.into_boxed_slice())
    }

    /// Create an indent block from a vector of elements.
    pub fn indent(elements: Vec<IRElement>) -> Self {
        IRElement::Indent(elements.into_boxed_slice())
    }

    /// Create a soft line break (becomes space when not breaking).
    pub fn soft_break() -> Self {
        IRElement::SoftLineBreak
    }

    /// Create a softest line break (becomes nothing when not breaking).
    pub fn softest_break() -> Self {
        IRElement::SoftestLineBreak
    }

    /// Create a hard line break.
    pub fn hard_break() -> Self {
        IRElement::HardLineBreak
    }

    /// Create a comment element.
    pub fn comment(text: impl Into<String>, is_doc: bool) -> Self {
        IRElement::Comment {
            text: text.into(),
            is_doc,
        }
    }
}
