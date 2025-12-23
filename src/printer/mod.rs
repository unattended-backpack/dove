//! Wadler-style printer for converting IR to formatted strings.
//!
//! This module implements the core printing logic based on Philip Wadler's
//! "A prettier printer" algorithm. The printer uses an iterative command queue
//! approach to process IR elements and make intelligent layout decisions.

use crate::ir_builder::ir::IRElement;

/// The two modes a group can be printed in.
#[derive(Clone, Copy, Debug, PartialEq)]
enum PrintMode {
    /// Try to fit everything on one line (SoftLineBreaks become spaces).
    Flat,
    /// Break at all SoftLineBreaks (they become newlines with proper indentation).
    Break,
}

/// A command for the printer's work queue.
#[derive(Debug)]
struct PrintCommand<'a> {
    mode: PrintMode,
    indent: usize,
    element: &'a IRElement,
}

/// Wadler-style printer that makes intelligent layout decisions.
pub struct Printer {
    /// Maximum line width before breaking.
    max_width: usize,
    /// Number of spaces per indentation level.
    indent_width: usize,
    /// Output buffer.
    buffer: String,
    /// Current column position (for width calculations).
    current_col: usize,
    /// Pending indentation to emit before next text (lazy indentation).
    /// None means no pending indent, Some(n) means emit n levels of indent before next text.
    pending_indent: Option<usize>,
}

impl Printer {
    /// Create a new printer with default settings.
    pub fn new() -> Self {
        Printer {
            max_width: 80,   // Standard line width
            indent_width: 2, // 2-space indentation
            buffer: String::new(),
            current_col: 0,
            pending_indent: None,
        }
    }

    /// Create a printer with custom settings.
    pub fn with_config(max_width: usize, indent_width: usize) -> Self {
        Printer {
            max_width,
            indent_width,
            buffer: String::new(),
            current_col: 0,
            pending_indent: None,
        }
    }

    /// Print IR elements to a formatted string.
    pub fn print(&mut self, ir: &[IRElement]) -> String {
        // Clear previous state
        self.buffer.clear();
        self.current_col = 0;
        self.pending_indent = None;

        // Initialize command queue with root elements
        let mut commands: Vec<PrintCommand> = ir
            .iter()
            .rev() // We'll use the Vec as a stack (LIFO)
            .map(|el| PrintCommand {
                mode: PrintMode::Break,
                indent: 0,
                element: el,
            })
            .collect();

        // Process commands until queue is empty
        while let Some(cmd) = commands.pop() {
            match cmd.element {
                IRElement::Text(text) => self.print_text(text),
                IRElement::SoftLineBreak => self.print_soft_line_break(cmd.mode, cmd.indent),
                IRElement::SoftestLineBreak => self.print_softest_line_break(cmd.mode, cmd.indent),
                IRElement::HardLineBreak => self.print_hard_line_break(cmd.indent),
                IRElement::Indent(elements) => {
                    // If there's a pending indent (from a HardLineBreak before this Indent),
                    // update it to the new indent level so content inside gets correct indentation
                    if self.pending_indent.is_some() {
                        self.pending_indent = Some(cmd.indent + 1);
                    }
                    // Push children with increased indent, maintaining mode
                    for el in elements.iter().rev() {
                        commands.push(PrintCommand {
                            mode: cmd.mode,
                            indent: cmd.indent + 1,
                            element: el,
                        });
                    }
                }
                IRElement::Group(elements) => {
                    // Check if the entire group fits on the current line
                    if self.group_fits(elements, cmd.indent) {
                        // Fits - render everything flat (soft breaks become spaces)
                        for el in elements.iter().rev() {
                            commands.push(PrintCommand {
                                mode: PrintMode::Flat,
                                indent: cmd.indent,
                                element: el,
                            });
                        }
                    } else {
                        // Doesn't fit - use fill semantics (greedy line filling)
                        self.print_fill(elements, cmd.indent);
                    }
                }
                IRElement::Comment { text, is_doc } => {
                    self.print_comment(text, *is_doc, cmd.indent);
                }
            }
        }

        // Ensure file ends with a newline
        if !self.buffer.ends_with('\n') {
            self.buffer.push('\n');
        }

        self.buffer.clone()
    }

    /// Check if a group fits on the current line.
    fn group_fits(&self, elements: &[IRElement], _indent: usize) -> bool {
        // Check if the group fits with a small margin for trailing content like ; or )
        match check_flat_layout_fits(elements, self.current_col, self.max_width) {
            Some(end_col) => end_col < self.max_width, // Strict: must be < not <=
            None => false,
        }
    }

    /// Flush any pending indentation before printing content.
    fn flush_pending_indent(&mut self) {
        if let Some(indent) = self.pending_indent.take() {
            let indent_str = " ".repeat(indent * self.indent_width);
            self.buffer.push_str(&indent_str);
            self.current_col = indent_str.chars().count();
        }
    }

    /// Print text and update current column.
    fn print_text(&mut self, text: &str) {
        self.flush_pending_indent();
        self.buffer.push_str(text);
        // Handle text that might contain newlines - Unicode-safe width calculation
        if let Some(last_newline) = text.rfind('\n') {
            let after_newline = &text[last_newline + 1..];
            self.current_col = after_newline.chars().count();
        } else {
            self.current_col += text.chars().count();
        }
    }

    /// Print a soft line break based on the current mode.
    fn print_soft_line_break(&mut self, mode: PrintMode, indent: usize) {
        match mode {
            PrintMode::Flat => {
                self.flush_pending_indent();
                self.buffer.push(' ');
                self.current_col += 1;
            }
            PrintMode::Break => {
                self.print_hard_line_break(indent);
            }
        }
    }

    /// Print a softest line break based on the current mode.
    /// Unlike SoftLineBreak, this becomes nothing (not a space) in flat mode.
    fn print_softest_line_break(&mut self, mode: PrintMode, indent: usize) {
        match mode {
            PrintMode::Flat => {
                // Do nothing - softest line break disappears in flat mode
            }
            PrintMode::Break => {
                self.print_hard_line_break(indent);
            }
        }
    }

    /// Print a hard line break with lazy indentation.
    /// The indentation is stored as pending and only emitted when actual content follows.
    fn print_hard_line_break(&mut self, indent: usize) {
        // Strip trailing whitespace from the current line before adding newline
        while self.buffer.ends_with(' ') {
            self.buffer.pop();
        }
        self.buffer.push('\n');
        self.pending_indent = Some(indent);
        // Set current_col to where we'll be after indent is flushed
        self.current_col = indent * self.indent_width;
    }

    /// Print a comment, deciding between line and block format based on length.
    fn print_comment(&mut self, text: &str, is_doc: bool, indent: usize) {
        let prefix = if is_doc { "///" } else { "//" };
        let line_comment = format!("{} {}", prefix, text);

        // Check if the text contains newlines - if so, preserve them in a block comment
        let has_newlines = text.contains('\n');

        // Check if it fits as a line comment (only if no newlines)
        let indent_width = indent * self.indent_width;
        if !has_newlines && indent_width + line_comment.len() <= self.max_width {
            self.print_text(&line_comment);
        } else if has_newlines {
            // Text has intentional newlines - preserve them in block comment
            let block_open = if is_doc { "/**" } else { "/*" };
            self.print_text(block_open);
            self.print_hard_line_break(indent + 1);

            // Process line by line, preserving original structure
            for (i, line) in text.lines().enumerate() {
                if i > 0 {
                    self.print_hard_line_break(indent + 1);
                }
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    self.print_text(trimmed);
                }
            }

            self.print_hard_line_break(indent);
            self.print_text("*/");
        } else {
            // Use block comment with word wrapping
            let block_open = if is_doc { "/**" } else { "/*" };
            self.print_text(block_open);
            self.print_hard_line_break(indent + 1);

            // Word wrap the text
            let words: Vec<&str> = text.split_whitespace().collect();
            let mut current_line_len = indent_width + self.indent_width; // inside block comment

            for (i, word) in words.iter().enumerate() {
                let word_len = word.chars().count();

                if i == 0 {
                    // First word
                    self.print_text(word);
                    current_line_len += word_len;
                } else if current_line_len + 1 + word_len <= self.max_width {
                    // Fits on current line with space
                    self.print_text(" ");
                    self.print_text(word);
                    current_line_len += 1 + word_len;
                } else {
                    // Need new line
                    self.print_hard_line_break(indent + 1);
                    self.print_text(word);
                    current_line_len = indent_width + self.indent_width + word_len;
                }
            }

            self.print_hard_line_break(indent);
            self.print_text("*/");
        }
    }

    /// Print elements in flat mode - SoftLineBreak becomes space, SoftestLineBreak disappears.
    fn print_flat(&mut self, elements: &[IRElement]) {
        for element in elements {
            match element {
                IRElement::Text(text) => self.print_text(text),
                IRElement::SoftLineBreak => {
                    self.flush_pending_indent();
                    self.buffer.push(' ');
                    self.current_col += 1;
                }
                IRElement::SoftestLineBreak => {
                    // Disappears in flat mode
                }
                IRElement::HardLineBreak => {
                    // Hard breaks still break even in flat mode
                    self.buffer.push('\n');
                    self.current_col = 0;
                }
                IRElement::Group(children) | IRElement::Indent(children) => {
                    self.print_flat(children);
                }
                IRElement::Comment { text, is_doc } => {
                    let prefix = if *is_doc { "///" } else { "//" };
                    self.print_text(&format!("{} {}", prefix, text));
                }
            }
        }
    }

    /// Print a fill group - greedily fits as many elements per line as possible.
    /// This is used for word-wrapped text where we want to maximize content per line.
    fn print_fill(&mut self, elements: &[IRElement], indent: usize) {
        let mut i = 0;
        // Track when SoftestLineBreak didn't break (greedy case) - used to skip
        // indent increment for immediately following Indent element
        let mut skip_next_indent = false;
        while i < elements.len() {
            match &elements[i] {
                IRElement::Text(text) => {
                    // Check if this text fits on the current line
                    let text_len = text.chars().count();
                    // Never break before statement terminators like ; or );
                    let is_terminator = text == ";" || text == ");" || text == "})";
                    if is_terminator || self.current_col + text_len <= self.max_width {
                        self.print_text(text);
                    } else {
                        // Doesn't fit - break to new line first
                        self.print_hard_line_break(indent);
                        self.print_text(text);
                    }
                    skip_next_indent = false;
                    i += 1;
                }
                IRElement::SoftLineBreak => {
                    // Look ahead to see if content until next break point fits
                    if let Some(chunk_len) = self.measure_next_chunk(&elements[i + 1..]) {
                        // +1 for the space that SoftLineBreak becomes in flat mode
                        if self.current_col + 1 + chunk_len <= self.max_width {
                            // Fits - render as space
                            self.flush_pending_indent();
                            self.buffer.push(' ');
                            self.current_col += 1;
                        } else {
                            // Doesn't fit - render as line break
                            self.print_hard_line_break(indent);
                        }
                    } else {
                        // No content, just render as space
                        self.flush_pending_indent();
                        self.buffer.push(' ');
                        self.current_col += 1;
                    }
                    skip_next_indent = false;
                    i += 1;
                }
                IRElement::SoftestLineBreak => {
                    // Only use greedy behavior when content has HardLineBreaks (like
                    // NamedFunctionCall/struct initializers). This allows "var = expr({"
                    // to stay on one line while the struct fields break internally.
                    // For content without HardLineBreaks, use original fill behavior.
                    if Self::has_hard_break(&elements[i + 1..]) {
                        let first_line_width = self.measure_until_hard_break(&elements[i + 1..]);
                        if first_line_width > 0
                            && self.current_col + first_line_width <= self.max_width
                        {
                            // First line content fits - render SoftestLineBreak as nothing
                            // Mark that the next Indent should not increment indent level
                            skip_next_indent = true;
                        } else {
                            // Doesn't fit - render as line break
                            self.print_hard_line_break(indent);
                            skip_next_indent = false;
                        }
                    } else {
                        // No HardLineBreaks - always break in fill mode (original behavior)
                        self.print_hard_line_break(indent);
                        skip_next_indent = false;
                    }
                    i += 1;
                }
                IRElement::HardLineBreak => {
                    self.print_hard_line_break(indent);
                    skip_next_indent = false;
                    i += 1;
                }
                IRElement::Group(children) => {
                    // Check if nested group fits on current line - if so, render flat
                    if self.group_fits(children, indent) {
                        self.print_flat(children);
                    } else {
                        self.print_fill(children, indent);
                    }
                    skip_next_indent = false;
                    i += 1;
                }
                IRElement::Indent(children) => {
                    // If skip_next_indent is set (from a non-breaking SoftestLineBreak),
                    // don't increment the indent level for this Indent
                    let effective_indent = if skip_next_indent { indent } else { indent + 1 };
                    // If there's a pending indent, update it to the new level
                    if self.pending_indent.is_some() {
                        self.pending_indent = Some(effective_indent);
                    }
                    // Recursively handle indents with the effective indent level
                    self.print_fill(children, effective_indent);
                    skip_next_indent = false;
                    i += 1;
                }
                IRElement::Comment { text, is_doc } => {
                    self.print_comment(text, *is_doc, indent);
                    skip_next_indent = false;
                    i += 1;
                }
            }
        }
    }

    /// Measure the flat width of content until the next SoftLineBreak.
    /// This is used for greedy fill decisions.
    fn measure_next_chunk(&self, elements: &[IRElement]) -> Option<usize> {
        let mut width = 0;
        for element in elements {
            match element {
                IRElement::Text(text) => width += text.chars().count(),
                IRElement::SoftLineBreak => break, // Stop at next break point
                IRElement::SoftestLineBreak => {
                    // SoftestLineBreak disappears in flat mode, continue measuring
                }
                IRElement::HardLineBreak => break, // Stop at hard breaks too
                IRElement::Group(children) | IRElement::Indent(children) => {
                    // Add the full flat width of nested groups
                    if let Some(end_col) = check_flat_layout_fits(children, 0, usize::MAX) {
                        width += end_col;
                    } else {
                        // Contains HardLineBreak, stop measuring
                        break;
                    }
                }
                IRElement::Comment { text, is_doc } => {
                    let prefix_len = if *is_doc { 4 } else { 3 }; // "/// " or "// "
                    width += prefix_len + text.chars().count();
                }
            }
        }
        if width > 0 {
            Some(width)
        } else {
            None
        }
    }

    /// Measure the width of content up to (but not including) the first HardLineBreak.
    /// This represents what would fit on the current line before a forced break.
    fn measure_until_hard_break(&self, elements: &[IRElement]) -> usize {
        let mut width = 0;
        for element in elements {
            match element {
                IRElement::Text(text) => width += text.chars().count(),
                IRElement::SoftLineBreak => width += 1, // space in flat mode
                IRElement::SoftestLineBreak => {}       // nothing in flat mode
                IRElement::HardLineBreak => break,      // stop at hard break
                IRElement::Group(children) | IRElement::Indent(children) => {
                    // Recursively measure until hard break in children
                    width += self.measure_until_hard_break(children);
                    // If children had a hard break, stop here
                    if Self::has_hard_break(children) {
                        break;
                    }
                }
                IRElement::Comment { text, is_doc } => {
                    let prefix_len = if *is_doc { 4 } else { 3 }; // "/// " or "// "
                    width += prefix_len + text.chars().count();
                }
            }
        }
        width
    }

    /// Check if elements contain a HardLineBreak anywhere.
    fn has_hard_break(elements: &[IRElement]) -> bool {
        for element in elements {
            match element {
                IRElement::HardLineBreak => return true,
                IRElement::Group(children) | IRElement::Indent(children) => {
                    if Self::has_hard_break(children) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }
}

impl Default for Printer {
    fn default() -> Self {
        Self::new()
    }
}

/// Recursively checks if a slice of IRElements can fit on a single line.
///
/// - `elements`: The IR to check.
/// - `start_col`: The column at which this check begins.
/// - `max_width`: The maximum allowed column width.
///
/// Returns `Some(end_col)` if the elements fit, where `end_col` is the
/// column after printing the last element. Returns `None` if they do not fit.
fn check_flat_layout_fits(
    elements: &[IRElement],
    start_col: usize,
    max_width: usize,
) -> Option<usize> {
    let mut current_col = start_col;

    for element in elements {
        if current_col > max_width {
            return None;
        }

        match element {
            IRElement::Text(s) => {
                current_col += s.chars().count(); // Unicode-safe width
            }
            IRElement::SoftLineBreak => {
                // In flat mode, soft line break becomes single space
                current_col += 1;
            }
            IRElement::SoftestLineBreak => {
                // In flat mode, softest line break becomes nothing (0 width)
            }
            IRElement::HardLineBreak => {
                // Hard breaks explicitly forbid flat layout
                return None;
            }
            IRElement::Indent(children) | IRElement::Group(children) => {
                // CRITICAL: Recursively check with current simulated column
                match check_flat_layout_fits(children, current_col, max_width) {
                    Some(new_col) => current_col = new_col,
                    None => return None,
                }
            }
            IRElement::Comment { text, is_doc } => {
                // Comment length: prefix + space + text
                let prefix_len = if *is_doc { 3 } else { 2 }; // "///" or "//"
                current_col += prefix_len + 1 + text.chars().count();
            }
        }
    }

    if current_col > max_width {
        None
    } else {
        Some(current_col)
    }
}

/// Legacy function for backward compatibility.
pub fn print_ir(ir_elements: &[IRElement]) -> String {
    let mut printer = Printer::new();
    printer.print(ir_elements)
}

#[cfg(test)]
pub mod tests;
