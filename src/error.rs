use std::fmt::Display;

use colorize::AnsiColor;

use crate::source_file::{Located, WithLocation};

pub enum Severity {
    Error,
    Warning,
    Hint
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Error => write!(f, "{}", "[Error]".bold().b_red()),
            Self::Warning => write!(f, "{}", "[Warning]".bold().b_yellow()),
            Self::Hint => write!(f, "{}", "[Hint]".bold().b_cyan())
        } 
    }
}

pub struct CompilerError {
    severity: Severity,

    message: String,
    hint: Option<String>,

    pub additional: Vec<Located<CompilerError>>
}

impl CompilerError {
    pub fn new(severity: Severity, message: String, hint: Option<String>, additional: Vec<Located<CompilerError>>) -> Self {
        Self {
            severity,
            message,
            hint,
            additional
        }
    }

    pub fn severity(&self) -> &Severity {
        &self.severity
    }

    pub fn message(&self) -> &String {
        &self.message
    }

    pub fn hint(&self) -> &Option<String> {
        &self.hint
    }
}

impl WithLocation for CompilerError {}

pub trait IntoCompilerError: Into<CompilerError> {}

impl IntoCompilerError for CompilerError {}

