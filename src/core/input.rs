use crate::core::error::*;
use content_inspector::ContentType;
use std::{
    fmt::Debug,
    fs::{File, metadata},
    io::{BufRead, BufReader, IsTerminal, stdin},
    path::PathBuf,
};

pub const DEFAULT_INPUT_FORMAT: InputFormat = InputFormat::Yaml;

#[derive(Debug)]
pub enum InputKind {
    Stdin,
    File(PathBuf),
}

#[derive(Debug)]
pub enum InputFormat {
    Json,
    Yaml,
    Unknown,
}

impl Default for InputFormat {
    fn default() -> Self {
        DEFAULT_INPUT_FORMAT
    }
}

impl InputFormat {
    pub fn from_extension(path: &PathBuf) -> Self {
        let ext = match path.extension() {
            Some(e) => e.to_str().unwrap_or("").to_lowercase(),
            None => "".to_string(),
        };

        match ext.as_str() {
            "json" => InputFormat::Json,
            "yaml" | "yml" => InputFormat::Yaml,
            _ => InputFormat::Unknown,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            InputFormat::Json => "json",
            InputFormat::Yaml => "yaml",
            InputFormat::Unknown => "unknown",
        }
    }
}

pub struct Input {
    pub reader: Box<dyn BufRead>,
    pub kind: InputKind,
    pub format: InputFormat,
    pub is_terminal: bool,
    pub content_type: Option<ContentType>,
    pub content: String,
}

impl Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Input")
            .field("kind", &self.kind)
            .field("format", &self.format)
            .field("is_terminal", &self.is_terminal)
            .field("content_type", &self.content_type)
            .finish()
    }
}

impl Input {
    pub fn new(path: &PathBuf) -> Result<Self> {
        let mut input;
        if path == &PathBuf::from("-") {
            input = Self::from_stdin();
        } else {
            match Self::from_file(path) {
                Ok(reader) => {
                    input = reader;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        input.inspect_content()?;
        Ok(input)
    }

    pub fn input_format(&self) -> &InputFormat {
        &self.format
    }

    pub fn buf_reader(&mut self) -> &mut dyn BufRead {
        &mut *self.reader
    }

    pub fn is_binary(&self) -> bool {
        match &self.content_type {
            Some(ct) => ct.is_binary(),
            None => false,
        }
    }

    pub fn from_stdin() -> Self {
        Input {
            reader: Box::new(BufReader::new(stdin())),
            kind: InputKind::Stdin,
            format: DEFAULT_INPUT_FORMAT,
            is_terminal: stdin().is_terminal(),
            content_type: None,
            content: String::new(),
        }
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
        _ = match check_file(path) {
            Ok(_) => (),
            Err(e) => {
                return Err(e.context(format!("error checking file: {}", path.display()).as_str()));
            }
        };

        let format = InputFormat::from_extension(path);

        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::new(SourceError::Io(e))
                    .context(format!("opening file: {}", path.display()).as_str())
                    .code(CODE_RUNTIME_ERROR)
                    .print_help());
            }
        };

        Ok(Input {
            reader: Box::new(BufReader::new(file)),
            kind: InputKind::File(path.to_path_buf()),
            format: format,
            is_terminal: false,
            content_type: None,
            content: String::new(),
        })
    }

    pub fn inspect_content(&mut self) -> Result<()> {
        let content = read_first_line(&mut *self.reader)?;

        if content.is_empty() {
            return Ok(());
        }

        let content_type = content_inspector::inspect(&content.as_bytes());
        self.content_type = Some(content_type);
        self.content = content;
        Ok(())
    }

    pub fn read_content(&mut self) -> Result<bool> {
        let reader = &mut *self.reader;
        for line in reader.lines() {
            match line {
                Ok(l) => self.content.push_str(&l),
                Err(e) => {
                    return Err(Error::new(SourceError::Io(e))
                        .context("reading input content")
                        .code(CODE_RUNTIME_ERROR));
                }
            }
        }

        Ok(true)
    }
}

// check_file checks if the file exists and is not a directory. Returns Ok(true) if it's a valid
// file, otherwise returns an appropriate Error.
fn check_file(path: &PathBuf) -> Result<bool> {
    match metadata(path) {
        Ok(metadata) => match metadata.is_dir() {
            // If it's a directory, return IsADirectory io::Error.
            true => Err(Error::new(SourceError::Io(std::io::Error::new(
                std::io::ErrorKind::IsADirectory,
                "Is a directory",
            )))
            .code(CODE_RUNTIME_ERROR)),
            false => Ok(true),
        },
        Err(e) => Err(Error::new(SourceError::Io(e))
            .code(CODE_RUNTIME_ERROR)
            .print_help()),
    }
}

pub fn read_first_line<R: BufRead>(mut reader: R) -> Result<String> {
    let mut content = String::new();
    match reader.read_line(&mut content) {
        Ok(_) => Ok(content),
        Err(e) => Err(Error::new(SourceError::Io(e))
            .context("reading first line")
            .code(CODE_RUNTIME_ERROR)),
    }
}
