use super::*;
use crate::collector::model::{CommentedCatchClause, CommentedElseBranch, CommentedStatement};
use super::yul::YulBlockExpectation;

// Statement expectations
pub struct StatementExpectation {
    statement_type: Option<String>,
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    nested_statements: Vec<StatementExpectation>,
    nested_standalone_comments: Vec<String>,
    else_branch: Option<Box<ElseBranchExpectation>>,
    catch_clauses: Vec<CatchClauseExpectation>,
    yul_block: Option<Box<YulBlockExpectation>>,
    // For loop specific expectations
    for_init_leading_comments: Vec<String>,
    for_init_trailing_comments: Vec<String>,
    for_condition_leading_comments: Vec<String>,
    for_condition_trailing_comments: Vec<String>,
    for_update_leading_comments: Vec<String>,
    for_update_trailing_comments: Vec<String>,
}

impl StatementExpectation {
    pub fn new() -> Self {
        Self {
            statement_type: None,
            leading_comments: vec![],
            trailing_comments: vec![],
            nested_statements: vec![],
            nested_standalone_comments: vec![],
            else_branch: None,
            catch_clauses: vec![],
            yul_block: None,
            for_init_leading_comments: vec![],
            for_init_trailing_comments: vec![],
            for_condition_leading_comments: vec![],
            for_condition_trailing_comments: vec![],
            for_update_leading_comments: vec![],
            for_update_trailing_comments: vec![],
        }
    }

    pub fn statement_type(mut self, stmt_type: &str) -> Self {
        self.statement_type = Some(stmt_type.to_string());
        self
    }

    pub fn nested_statement<F>(mut self, f: F) -> Self
    where
        F: FnOnce(StatementExpectation) -> StatementExpectation,
    {
        self.nested_statements.push(f(StatementExpectation::new()));
        self
    }

    pub fn has_nested_statements(mut self, count: usize) -> Self {
        // This is just a marker - the actual check happens in assert_matches
        self
    }

    pub fn nested_standalone_comment(mut self, comment: &str) -> Self {
        self.nested_standalone_comments.push(comment.to_string());
        self
    }

    pub fn else_branch<F>(mut self, f: F) -> Self
    where
        F: FnOnce(ElseBranchExpectation) -> ElseBranchExpectation,
    {
        self.else_branch = Some(Box::new(f(ElseBranchExpectation::new())));
        self
    }

    pub fn catch_clause<F>(mut self, f: F) -> Self
    where
        F: FnOnce(CatchClauseExpectation) -> CatchClauseExpectation,
    {
        self.catch_clauses.push(f(CatchClauseExpectation::new()));
        self
    }

    pub fn yul_block<F>(mut self, f: F) -> Self
    where
        F: FnOnce(YulBlockExpectation) -> YulBlockExpectation,
    {
        self.yul_block = Some(Box::new(f(YulBlockExpectation::new())));
        self
    }

    // For loop component comment methods
    pub fn for_init_leading(mut self, comment: &str) -> Self {
        self.for_init_leading_comments.push(comment.to_string());
        self
    }

    pub fn for_init_trailing(mut self, comment: &str) -> Self {
        self.for_init_trailing_comments.push(comment.to_string());
        self
    }

    pub fn for_condition_leading(mut self, comment: &str) -> Self {
        self.for_condition_leading_comments
            .push(comment.to_string());
        self
    }

    pub fn for_condition_trailing(mut self, comment: &str) -> Self {
        self.for_condition_trailing_comments
            .push(comment.to_string());
        self
    }

    pub fn for_update_leading(mut self, comment: &str) -> Self {
        self.for_update_leading_comments.push(comment.to_string());
        self
    }

    pub fn for_update_trailing(mut self, comment: &str) -> Self {
        self.for_update_trailing_comments.push(comment.to_string());
        self
    }

    pub fn assert_matches(&self, stmt: &CommentedStatement, context: &str) {
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

        // Check nested statements if any
        let nested_count = stmt.nested_statements.as_ref().map_or(0, |n| n.len());
        assert_eq!(
            self.nested_statements.len(),
            nested_count,
            "{}: expected {} nested statements, found {}",
            context,
            self.nested_statements.len(),
            nested_count
        );

        if let Some(nested) = &stmt.nested_statements {
            for (i, nested_exp) in self.nested_statements.iter().enumerate() {
                nested_exp.assert_matches(&nested[i], &format!("{}.nested[{}]", context, i));
            }
        }

        // Check nested standalone comments
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

        // Check else branch if present
        if self.else_branch.is_some() || stmt.else_branch.is_some() {
            assert!(
                self.else_branch.is_some() && stmt.else_branch.is_some(),
                "{}: else branch expectation mismatch",
                context
            );

            if let (Some(expected_else), Some(actual_else)) = (&self.else_branch, &stmt.else_branch)
            {
                expected_else.assert_matches(actual_else, &format!("{}.else", context));
            }
        }

        // Check catch clauses if present
        let catch_count = stmt.catch_clauses.as_ref().map_or(0, |c| c.len());
        assert_eq!(
            self.catch_clauses.len(),
            catch_count,
            "{}: expected {} catch clauses, found {}",
            context,
            self.catch_clauses.len(),
            catch_count
        );

        if let Some(catch_clauses) = &stmt.catch_clauses {
            for (i, expected_catch) in self.catch_clauses.iter().enumerate() {
                expected_catch
                    .assert_matches(&catch_clauses[i], &format!("{}.catch[{}]", context, i));
            }
        }

        // Check for loop component comments if present
        if !self.for_init_leading_comments.is_empty()
            || !self.for_init_trailing_comments.is_empty()
            || !self.for_condition_leading_comments.is_empty()
            || !self.for_condition_trailing_comments.is_empty()
            || !self.for_update_leading_comments.is_empty()
            || !self.for_update_trailing_comments.is_empty()
        {
            // Get the for loop component comments from the statement
            if let Some(for_comments) = &stmt.for_component_comments {
                // Check init comments
                assert_comments(
                    &for_comments.init_leading,
                    &self.for_init_leading_comments,
                    &format!("{}.for_init.leading", context),
                );
                assert_comments(
                    &for_comments.init_trailing,
                    &self.for_init_trailing_comments,
                    &format!("{}.for_init.trailing", context),
                );

                // Check condition comments
                assert_comments(
                    &for_comments.condition_leading,
                    &self.for_condition_leading_comments,
                    &format!("{}.for_condition.leading", context),
                );
                assert_comments(
                    &for_comments.condition_trailing,
                    &self.for_condition_trailing_comments,
                    &format!("{}.for_condition.trailing", context),
                );

                // Check update comments
                assert_comments(
                    &for_comments.update_leading,
                    &self.for_update_leading_comments,
                    &format!("{}.for_update.leading", context),
                );
                assert_comments(
                    &for_comments.update_trailing,
                    &self.for_update_trailing_comments,
                    &format!("{}.for_update.trailing", context),
                );
            } else {
                panic!(
                    "{}: Expected for loop component comments but none found",
                    context
                );
            }
        }

        // Check YUL block if present
        if self.yul_block.is_some() || stmt.yul_block.is_some() {
            assert!(
                self.yul_block.is_some() && stmt.yul_block.is_some(),
                "{}: YUL block expectation mismatch",
                context
            );

            if let (Some(expected_yul), Some(actual_yul)) = (&self.yul_block, &stmt.yul_block) {
                expected_yul.assert_matches(actual_yul, &format!("{}.yul", context));
            }
        }
    }
}

impl CommentExpectation for StatementExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// Else branch expectations
pub struct ElseBranchExpectation {
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    nested_statements: Vec<StatementExpectation>,
    nested_standalone_comments: Vec<String>,
}

impl ElseBranchExpectation {
    pub fn new() -> Self {
        Self {
            leading_comments: vec![],
            trailing_comments: vec![],
            nested_statements: vec![],
            nested_standalone_comments: vec![],
        }
    }

    pub fn nested_statement<F>(mut self, f: F) -> Self
    where
        F: FnOnce(StatementExpectation) -> StatementExpectation,
    {
        self.nested_statements.push(f(StatementExpectation::new()));
        self
    }

    pub fn nested_standalone_comment(mut self, comment: &str) -> Self {
        self.nested_standalone_comments.push(comment.to_string());
        self
    }

    pub fn assert_matches(&self, else_branch: &CommentedElseBranch, context: &str) {
        // Check comments
        assert_comments(
            &else_branch.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &else_branch.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check nested statements if any
        let nested_count = else_branch
            .nested_statements
            .as_ref()
            .map_or(0, |n| n.len());
        assert_eq!(
            self.nested_statements.len(),
            nested_count,
            "{}: expected {} nested statements, found {}",
            context,
            self.nested_statements.len(),
            nested_count
        );

        if let Some(nested) = &else_branch.nested_statements {
            for (i, nested_exp) in self.nested_statements.iter().enumerate() {
                nested_exp.assert_matches(&nested[i], &format!("{}.nested[{}]", context, i));
            }
        }

        // Check nested standalone comments
        let standalone_count = else_branch
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

        if let Some(standalone) = &else_branch.nested_standalone_comments {
            for (i, expected_comment) in self.nested_standalone_comments.iter().enumerate() {
                assert_comment_text(
                    &standalone[i],
                    expected_comment,
                    &format!("{}.standalone[{}]", context, i),
                );
            }
        }
    }
}

impl CommentExpectation for ElseBranchExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }
    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}

// Catch clause expectation for try-catch statements
pub struct CatchClauseExpectation {
    clause_type: Option<String>, // e.g., "Error", "Panic", or None for catch-all
    leading_comments: Vec<String>,
    trailing_comments: Vec<String>,
    body_statements: Vec<StatementExpectation>,
    body_standalone_comments: Vec<String>,
}

impl CatchClauseExpectation {
    pub fn new() -> Self {
        Self {
            clause_type: None,
            leading_comments: vec![],
            trailing_comments: vec![],
            body_statements: vec![],
            body_standalone_comments: vec![],
        }
    }

    pub fn clause_type(mut self, clause_type: &str) -> Self {
        self.clause_type = Some(clause_type.to_string());
        self
    }

    pub fn body_statement<F>(mut self, f: F) -> Self
    where
        F: FnOnce(StatementExpectation) -> StatementExpectation,
    {
        self.body_statements.push(f(StatementExpectation::new()));
        self
    }

    pub fn body_standalone_comment(mut self, comment: &str) -> Self {
        self.body_standalone_comments.push(comment.to_string());
        self
    }

    pub fn assert_matches(&self, catch: &CommentedCatchClause, context: &str) {
        // Check comments
        assert_comments(
            &catch.leading_comments,
            &self.leading_comments,
            &format!("{}.leading", context),
        );
        assert_comments(
            &catch.trailing_comments,
            &self.trailing_comments,
            &format!("{}.trailing", context),
        );

        // Check body statements
        assert_eq!(
            self.body_statements.len(),
            catch.body_statements.len(),
            "{}: expected {} body statements, found {}",
            context,
            self.body_statements.len(),
            catch.body_statements.len()
        );

        for (i, expected_stmt) in self.body_statements.iter().enumerate() {
            expected_stmt.assert_matches(
                &catch.body_statements[i],
                &format!("{}.body[{}]", context, i),
            );
        }

        // Check body standalone comments
        assert_eq!(
            self.body_standalone_comments.len(),
            catch.body_standalone_comments.len(),
            "{}: expected {} body standalone comments, found {}",
            context,
            self.body_standalone_comments.len(),
            catch.body_standalone_comments.len()
        );

        for (i, expected_comment) in self.body_standalone_comments.iter().enumerate() {
            assert_comment_text(
                &catch.body_standalone_comments[i],
                expected_comment,
                &format!("{}.body_standalone[{}]", context, i),
            );
        }
    }
}

impl CommentExpectation for CatchClauseExpectation {
    fn leading_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.leading_comments
    }

    fn trailing_comments_mut(&mut self) -> &mut Vec<String> {
        &mut self.trailing_comments
    }
}