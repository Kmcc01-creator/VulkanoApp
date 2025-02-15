//! Error types for the version tracking tools.

use std::error::Error as StdError;
use std::fmt;

/// Result type used throughout the crate
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type that encompasses all possible errors
#[derive(Debug)]
pub enum Error {
    /// AST analysis errors
    Analysis(#[from] ast_analyzer::Error),

    /// Code generation errors
    Generation(#[from] code_generator::Error),

    /// Parsing errors
    Parse(#[from] syn::Error),

    /// Template errors
    Template {
        /// Error message
        message: String,
        /// Additional context
        context: Option<String>,
    },

    /// Pattern matching errors
    Pattern {
        /// Error message
        message: String,
        /// Pattern name
        pattern: Option<String>,
    },

    /// Transform errors
    Transform {
        /// Error message
        message: String,
        /// Transform name
        transform: Option<String>,
    },

    /// Configuration errors
    Config {
        /// Error message
        message: String,
        /// Configuration key
        key: Option<String>,
    },

    /// I/O errors
    Io(#[from] std::io::Error),

    /// Scripting errors
    #[cfg(feature = "scripting")]
    Script {
        /// Error message
        message: String,
        /// Script line number
        line: Option<usize>,
    },

    /// Version tracking errors
    VersionTracking {
        /// Error message
        message: String,
        /// Version information
        version: Option<String>,
    },

    /// Other errors
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Analysis(e) => write!(f, "Analysis error: {}", e),
            Error::Generation(e) => write!(f, "Generation error: {}", e),
            Error::Parse(e) => write!(f, "Parse error: {}", e),
            Error::Template { message, context } => {
                write!(f, "Template error: {}", message)?;
                if let Some(ctx) = context {
                    write!(f, " (context: {})", ctx)?;
                }
                Ok(())
            }
            Error::Pattern { message, pattern } => {
                write!(f, "Pattern error: {}", message)?;
                if let Some(p) = pattern {
                    write!(f, " (pattern: {})", p)?;
                }
                Ok(())
            }
            Error::Transform { message, transform } => {
                write!(f, "Transform error: {}", message)?;
                if let Some(t) = transform {
                    write!(f, " (transform: {})", t)?;
                }
                Ok(())
            }
            Error::Config { message, key } => {
                write!(f, "Configuration error: {}", message)?;
                if let Some(k) = key {
                    write!(f, " (key: {})", k)?;
                }
                Ok(())
            }
            Error::Io(e) => write!(f, "I/O error: {}", e),
            #[cfg(feature = "scripting")]
            Error::Script { message, line } => {
                write!(f, "Script error: {}", message)?;
                if let Some(l) = line {
                    write!(f, " (line: {})", l)?;
                }
                Ok(())
            }
            Error::VersionTracking { message, version } => {
                write!(f, "Version tracking error: {}", message)?;
                if let Some(v) = version {
                    write!(f, " (version: {})", v)?;
                }
                Ok(())
            }
            Error::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Analysis(e) => Some(e),
            Error::Generation(e) => Some(e),
            Error::Parse(e) => Some(e),
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Template {
            message: "test error".to_string(),
            context: Some("test context".to_string()),
        };
        assert!(err.to_string().contains("test error"));
        assert!(err.to_string().contains("test context"));
    }

    #[test]
    fn test_error_conversion() {
        let err: Error = "test error".into();
        assert!(matches!(err, Error::Other(_)));
    }
}
