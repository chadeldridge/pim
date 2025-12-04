use std::io::Write;
use thiserror::Error;

// Barrowed from eza.
/// Exit code for successful execution.
pub const CODE_SUCCESS: i32 = 0;

/// Exit code for when there was at least one I/O error during execution.
pub const CODE_RUNTIME_ERROR: i32 = 1;

/// Exit code for when the command-line options are invalid.
pub const CODE_OPTIONS_ERROR: i32 = 3;

/// Exit code for missing file permissions
pub const CODE_PERMISSION_DENIED: i32 = 13;

// Barrowed heavily from bat.

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SourceError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] ::serde_json::Error),
    #[error(transparent)]
    SerdeYaml(#[from] ::serde_yaml::Error),
    #[error("Unsupported input format: {0}")]
    UnsupportedInputFormat(String),
    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
    #[error("Invalid input source: {0}")]
    InvalidInputSource(String),
    #[error("{0}")]
    Msg(String),
}

impl From<&'static str> for SourceError {
    fn from(s: &'static str) -> Self {
        SourceError::Msg(s.to_owned())
    }
}

impl From<String> for SourceError {
    fn from(s: String) -> Self {
        SourceError::Msg(s)
    }
}

#[derive(Debug)]
pub struct Error {
    pub code: Option<i32>,
    pub context: String,
    pub print_help: bool,
    pub source: SourceError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.context.is_empty() {
            write!(f, "{}", self.source)
        } else {
            write!(f, "{}\n{}", self.context, self.source)
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

impl Error {
    pub fn new(source: SourceError) -> Self {
        Error {
            code: None,
            context: String::new(),
            print_help: false,
            source,
        }
    }

    pub fn context(mut self, context: &str) -> Self {
        if !context.is_empty() {
            self.context = context.to_owned();
        } else {
            self.context = format!("{}\n{}", context, self.context);
        }

        self
    }

    pub fn print_help(mut self) -> Self {
        self.print_help = true;
        self
    }

    pub fn with_print_help(mut self, print_help: bool) -> Self {
        self.print_help = print_help;
        self
    }

    pub fn code(mut self, code: i32) -> Self {
        self.code = Some(code);
        self
    }

    pub fn no_code(mut self) -> Self {
        self.code = None;
        self
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn handle_error(error: &Error, output: &mut dyn Write) {
    match &error.source {
        SourceError::Io(io_err) if io_err.kind() == std::io::ErrorKind::BrokenPipe => {
            // Silent exit on broken pipe
            ::std::process::exit(0);
        }
        SourceError::SerdeJson(_) | SourceError::SerdeYaml(_) => {
            writeln!(output, "Error while parsing file: {error}",).ok();
        }
        _ => {
            writeln!(output, "{error}",).ok();
        }
    }
}
