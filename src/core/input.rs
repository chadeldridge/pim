use crate::core::error::*;
use crate::core::io::*;
use content_inspector::ContentType;
use log::{debug, warn};
use std::{
    fmt::Debug,
    fs::{Metadata, metadata},
    io::{IsTerminal, stdin},
    path::{Path, PathBuf},
};

pub const DEFAULT_INPUT_FORMAT: InputFormat = InputFormat::Yaml;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputKind {
    Stdin,
    File(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub fn from_extension(path: &Path) -> Self {
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
    reader: Reader,
    kind: InputKind,
    format: InputFormat,
    is_terminal: bool,
    content_type: Option<ContentType>,
    content: String,
    metadata: Option<Metadata>,
}

impl Debug for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Input")
            .field("reader", &"Reader {...}")
            .field("kind", &self.kind)
            .field("format", &self.format)
            .field("is_terminal", &self.is_terminal)
            .field("content_type", &self.content_type)
            .field("content", &self.content)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl Input {
    pub fn new(path: &Path) -> Result<Self> {
        let mut input;
        let reader = Reader::new(path)?;
        match reader {
            Reader::Stdin(_) => {
                input = Self::from_stdin(reader);
            }
            Reader::File(_) => {
                input = Self::from_file(&path.to_path_buf(), reader)?;
            }
            Reader::None => {
                return Err(Error::new(SourceError::InvalidInputSource(
                    path.display().to_string(),
                ))
                .set_context("Input source is None")
                .set_code(CODE_RUNTIME_ERROR));
            }
        }

        input.inspect_content()?;
        Ok(input)
    }

    pub fn format(&self) -> &InputFormat {
        &self.format
    }

    pub fn reader(&self) -> &Reader {
        &self.reader
    }

    pub fn mut_reader(&mut self) -> &mut Reader {
        &mut self.reader
    }

    pub fn kind(&self) -> &InputKind {
        &self.kind
    }

    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    pub fn content(&self) -> &String {
        &self.content
    }

    pub fn is_binary(&self) -> bool {
        match &self.content_type {
            Some(ct) => ct.is_binary(),
            None => false,
        }
    }

    pub fn is_dir(&self) -> bool {
        match &self.kind {
            InputKind::File(_) => is_dir(&self.metadata),
            InputKind::Stdin => false,
        }
    }

    pub fn from_stdin(reader: Reader) -> Self {
        debug!("Creating Input from stdin");
        Input {
            reader,
            kind: InputKind::Stdin,
            format: DEFAULT_INPUT_FORMAT,
            is_terminal: stdin().is_terminal(),
            content_type: None,
            content: String::new(),
            metadata: None,
        }
    }

    pub fn from_file(path: &PathBuf, reader: Reader) -> Result<Self> {
        debug!("Creating Input from file: {}", path.display());
        let metadata = metadata(path).map_err(|e| {
            Error::new(SourceError::Io(e))
                .set_context(format!("error checking file: {}", path.display()).as_str())
                .set_code(CODE_RUNTIME_ERROR)
                .print_help()
        })?;
        debug!("File metadata obtained: {:?}", metadata);

        let format = InputFormat::from_extension(path);
        debug!(
            "Determined input format as '{}' from file extension",
            format.as_str()
        );

        Ok(Input {
            reader,
            kind: InputKind::File(path.to_path_buf()),
            format,
            is_terminal: false,
            content_type: None,
            content: String::new(),
            metadata: Some(metadata),
        })
    }

    pub fn inspect_content(&mut self) -> Result<()> {
        debug!("Inspecting content type for input: {:?}", self.kind);
        //let content = read_first_line(&mut self.reader)?;
        let content = match &mut self.reader {
            Reader::Stdin(r) => read_first_line(r)?,
            Reader::File(file) => read_first_line(file)?,
            Reader::None => {
                return Err(Error::new(SourceError::InvalidInputSource(
                    "No reader available (None), skipping content inspection".to_string(),
                ))
                .set_context("No reader available (None) during content inspection")
                .set_code(CODE_RUNTIME_ERROR));
            }
        };

        if content.is_empty() {
            warn!("File {:?} is empty", self.kind);
            return Ok(());
        }

        let content_type = content_inspector::inspect(content.as_bytes());
        self.content_type = Some(content_type);
        self.content = content;
        debug!(
            "Content type inspected: {:?} based on content: {}",
            self.content_type, self.content
        );
        Ok(())
    }
}
