use crate::core::error::*;
use std::{
    fmt::Debug,
    fs::File,
    io::{IsTerminal, Write},
    path::PathBuf,
};

pub const DEFAULT_OUTPUT_FORMAT: OutputFormat = OutputFormat::Json;

#[derive(Debug)]
pub enum OutputKind {
    Stdout,
    File(PathBuf),
}

#[derive(Debug)]
pub enum OutputFormat {
    Json,
    Yaml,
}

impl Default for OutputFormat {
    fn default() -> Self {
        DEFAULT_OUTPUT_FORMAT
    }
}

impl OutputFormat {
    pub fn as_str(&self) -> &str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Yaml => "yaml",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Yaml => "yml",
        }
    }
}

pub struct Output {
    pub path: PathBuf,
    pub writer: Box<dyn std::io::Write>,
    pub kind: OutputKind,
    pub format: OutputFormat,
    pub pretty: bool,
}

impl Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Output")
            .field("kind", &self.kind)
            .field("format", &self.format)
            .field("pretty", &self.pretty)
            .finish()
    }
}

impl Output {
    pub fn new(path: &PathBuf, format: OutputFormat) -> Result<Self> {
        // The only time we don't pretty print is when writing to non-terminal stdout.
        let (writer, kind, pretty): (Box<dyn std::io::Write>, OutputKind, bool) =
            if path.to_str().unwrap_or("") == "<stdout>" {
                let is_terminal = std::io::stdout().is_terminal();
                (Box::new(std::io::stdout()), OutputKind::Stdout, is_terminal)
            } else {
                let file = match File::create(path) {
                    Ok(f) => f,
                    Err(e) => {
                        return Err(Error::new(SourceError::Io(e)).context(&format!(
                            "Failed to create output file: {}",
                            path.display()
                        )));
                    }
                };
                // Files should always be written with pretty printing for readability.
                (Box::new(file), OutputKind::File(path.clone()), true)
            };

        Ok(Output {
            path: path.clone(),
            writer,
            kind,
            format,
            pretty,
        })
    }

    pub fn output_format(&self) -> &OutputFormat {
        &self.format
    }

    pub fn buf_writer(&mut self) -> &mut dyn std::io::Write {
        &mut *self.writer
    }

    pub fn write<T: serde::Serialize>(&mut self, content: &T) -> Result<()> {
        if self.pretty {
            pretty(&mut self.writer, content, &self.format)
        } else {
            raw(&mut self.writer, content, &self.format)
        }
    }
}

// Write unpretty formatted output to the file or stdout.
pub fn raw<T: serde::Serialize>(
    writer: &mut Box<dyn Write>,
    content: &T,
    format: &OutputFormat,
) -> Result<()> {
    let data = match format {
        OutputFormat::Json => serde_json::to_string(content).map_err(|e| {
            Error::new(SourceError::SerdeJson(e))
                .context("Failed to serialize to JSON")
                .code(CODE_RUNTIME_ERROR)
        })?,
        OutputFormat::Yaml => serde_yaml::to_string(content).map_err(|e| {
            Error::new(SourceError::SerdeYaml(e))
                .context("Failed to serialize to YAML")
                .code(CODE_RUNTIME_ERROR)
        })?,
    };

    match writer.write_all(data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(SourceError::Io(e)).context("Failed to write output")),
    }
}

// Write pretty formatted output to the file or stdout.
pub fn pretty<T: serde::Serialize>(
    writer: &mut Box<dyn Write>,
    content: &T,
    format: &OutputFormat,
) -> Result<()> {
    let data = match format {
        OutputFormat::Json => serde_json::to_string_pretty(content).map_err(|e| {
            Error::new(SourceError::SerdeJson(e))
                .context("Failed to serialize to JSON")
                .code(CODE_RUNTIME_ERROR)
        })?,
        OutputFormat::Yaml => serde_yaml::to_string(content).map_err(|e| {
            Error::new(SourceError::SerdeYaml(e))
                .context("Failed to serialize to YAML")
                .code(CODE_RUNTIME_ERROR)
        })?,
    };

    match writer.write_all(data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(SourceError::Io(e))
            .context("Failed to write output")
            .code(CODE_RUNTIME_ERROR)),
    }
}
