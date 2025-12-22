use super::*;
use crate::collector::model::{CollectedYulBlock, CommentedYulStatement, CommentedYulCase};
use solang_parser::pt::Comment;
use solang_parser::pt::YulStatement;

// YUL block expectation for assembly blocks
pub struct YulBlockExpectation {
    statements: Vec<YulStatementExpectation>,
    standalone_comments: Vec<String>,
}

impl YulBlockExpectation {
    pub fn new() -> Self {
        Self {
            statements: vec![],
            standalone_comments: vec![],
        }
    }

    pub fn statement<F>(mut self, f: F) -> Self
    where
        F: FnOnce(YulStatementExpectation) -> YulStatementExpectation,
    {
        self.statements.push(f(YulStatementExpectation::new()));
        self
    }

    pub fn standalone_comment(mut self, comment: &str) -> Self {
        self.standalone_comments.push(comment.to_string());
        self
    }

    pub fn assert_matches(&self, yul_block: &CollectedYulBlock, context: &str) {
        // Check statements
        assert_eq!(
            self.statements.len(),
            yul_block.statements.len(),
            "{}: expected {} YUL statements, found {}",
            context,
            self.statements.len(),
            yul_block.statements.len()
        );

        for (i, expected_stmt) in self.statements.iter().enumerate() {
            expected_stmt.assert_matches(
                &yul_block.statements[i],
                &format!("{}.stmt[{}]", context, i),
            );
        }

        // Check standalone comments
        assert_eq!(
            self.standalone_comments.len(),
            yul_block.standalone_comments.len(),
            "{}: expected {} standalone comments, found {}",
            context,
            self.standalone_comments.len(),
            yul_block.standalone_comments.len()
        );

        for (i, expected_comment) in self.standalone_comments.iter().enumerate() {
            assert_comment_text(
                &yul_block.standalone_comments[i],
                expected_comment,
                &format!("{}.standalone[{}]", context, i),
            );
        }
    }
}

// YUL statement expectation
pub struct YulStatementExpectation {
    statement_type: Option<String>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    nested_statements: Vec<YulStatementExpectation>,
    nested_standalone_comments: Vec<String>,
    switch_cases: Vec<YulSwitchCaseExpectation>,
}

impl YulStatementExpectation {
    fn new() -> Self {
        Self {
            statement_type: None,
            leading_comments: vec![],
            trailing_comments: vec![],
            nested_statements: vec![],
            nested_standalone_comments: vec![],
            switch_cases: vec![],
        }
    }

    pub fn statement_type(mut self, stmt_type: &str) -> Self {
        self.statement_type = Some(stmt_type.to_string());
        self
    }

    pub fn nested_statement<F>(mut self, f: F) -> Self
    where
        F: FnOnce(YulStatementExpectation) -> YulStatementExpectation,
    {
        self.nested_statements
            .push(f(YulStatementExpectation::new()));
        self
    }

    pub fn nested_standalone_comment(mut self, comment: &str) -> Self {
        self.nested_standalone_comments.push(comment.to_string());
        self
    }

    pub fn switch_case<F>(mut self, f: F) -> Self
    where
        F: FnOnce(YulSwitchCaseExpectation) -> YulSwitchCaseExpectation,
    {
        self.switch_cases.push(f(YulSwitchCaseExpectation::new()));
        self
    }

    fn assert_matches(&self, stmt: &CommentedYulStatement, context: &str) {
        // Check comments
        assert_comments(
            &stmt.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &stmt.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check nested statements if any (skip for switch statements as they use switch_cases)
        if self.switch_cases.is_empty() {
            let nested_count = stmt.nested_statements.as_ref().map_or(0, |n| n.len());
            assert_eq!(
                self.nested_statements.len(),
                nested_count,
                "{}: expected {} nested YUL statements, found {}",
                context,
                self.nested_statements.len(),
                nested_count
            );

            if let Some(nested) = &stmt.nested_statements {
                for (i, nested_exp) in self.nested_statements.iter().enumerate() {
                    nested_exp.assert_matches(&nested[i], &format!("{}.nested[{}]", context, i));
                }
            }
        }

        // Check nested standalone comments (skip for switch statements as they use switch_cases)
        if self.switch_cases.is_empty() {
            let standalone_count = stmt
                .nested_standalone_comments
                .as_ref()
                .map_or(0, |c| c.len());
            assert_eq!(
                self.nested_standalone_comments.len(),
                standalone_count,
                "{}: expected {} nested standalone comments, found {}",
                context,
                self.nested_standalone_comments.len(),
                standalone_count
            );

            if let Some(standalone) = &stmt.nested_standalone_comments {
                for (i, expected_comment) in self.nested_standalone_comments.iter().enumerate() {
                    assert_comment_text(
                        &standalone[i],
                        expected_comment,
                        &format!("{}.standalone[{}]", context, i),
                    );
                }
            }
        }

        // Check switch cases if it's a switch statement
        if !self.switch_cases.is_empty() {
            match &stmt.statement {
                YulStatement::Switch(_) => {
                    let actual_cases = stmt.switch_cases.as_ref().expect(&format!(
                        "{}: switch statement should have switch_cases",
                        context
                    ));

                    assert_eq!(
                        self.switch_cases.len(),
                        actual_cases.len(),
                        "{}: expected {} switch cases, found {}",
                        context,
                        self.switch_cases.len(),
                        actual_cases.len()
                    );

                    for (i, expected_case) in self.switch_cases.iter().enumerate() {
                        expected_case
                            .assert_matches(&actual_cases[i], &format!("{}.case[{}]", context, i));
                    }
                }
                _ => panic!(
                    "{}: expected switch statement but found {:?}",
                    context, stmt.statement
                ),
            }
        }
    }
}

impl CommentExpectation for YulStatementExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }

    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// YUL switch case expectation
pub struct YulSwitchCaseExpectation {
    case_type: Option<String>, // "default" or the case value
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    body_statements: Vec<YulStatementExpectation>,
    body_standalone_comments: Vec<String>,
}

impl YulSwitchCaseExpectation {
    fn new() -> Self {
        Self {
            case_type: None,
            leading_comments: vec![],
            trailing_comments: vec![],
            body_statements: vec![],
            body_standalone_comments: vec![],
        }
    }

    pub fn case_type(mut self, case_type: &str) -> Self {
        self.case_type = Some(case_type.to_string());
        self
    }

    pub fn body_statement<F>(mut self, f: F) -> Self
    where
        F: FnOnce(YulStatementExpectation) -> YulStatementExpectation,
    {
        self.body_statements.push(f(YulStatementExpectation::new()));
        self
    }

    pub fn body_standalone_comment(mut self, comment: &str) -> Self {
        self.body_standalone_comments.push(comment.to_string());
        self
    }

    fn assert_matches(&self, case: &CommentedYulCase, context: &str) {
        // Check comments
        assert_comments(
            &case.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &case.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check body statements
        assert_eq!(
            self.body_statements.len(),
            case.body_statements.len(),
            "{}: expected {} body statements, found {}",
            context,
            self.body_statements.len(),
            case.body_statements.len()
        );

        for (i, expected_stmt) in self.body_statements.iter().enumerate() {
            expected_stmt.assert_matches(
                &case.body_statements[i],
                &format!("{}.body[{}]", context, i),
            );
        }

        // Check body standalone comments
        assert_eq!(
            self.body_standalone_comments.len(),
            case.body_standalone_comments.len(),
            "{}: expected {} body standalone comments, found {}",
            context,
            self.body_standalone_comments.len(),
            case.body_standalone_comments.len()
        );

        for (i, expected_comment) in self.body_standalone_comments.iter().enumerate() {
            assert_comment_text(
                &case.body_standalone_comments[i],
                expected_comment,
                &format!("{}.body_standalone[{}]", context, i),
            );
        }
    }
}

impl CommentExpectation for YulSwitchCaseExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }

    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}