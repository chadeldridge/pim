use crate::core::error::*;
use crate::core::io::*;
use log::debug;
use std::{
    cmp::Ordering,
    fmt::Debug,
    io::{IsTerminal, Write},
    path::{Path, PathBuf},
};

pub const DEFAULT_OUTPUT_FORMAT: OutputFormat = OutputFormat::Json;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OutputKind {
    Stdout,
    File(PathBuf),
    Directory(PathBuf),
}

impl OutputKind {
    pub fn new(path: &Path) -> Self {
        if path.to_str().unwrap_or("<stdout>") == "<stdout>" {
            OutputKind::Stdout
        } else if path.is_dir() {
            OutputKind::Directory(path.to_path_buf())
        } else {
            OutputKind::File(path.to_path_buf())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    path: PathBuf,
    writer: Box<dyn Write>,
    kind: OutputKind,
    format: OutputFormat,
    pretty: bool,
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

impl Eq for Output {}
impl PartialEq for Output {
    fn eq(&self, other: &Self) -> bool {
        (&self.path, &self.kind, &self.format, self.pretty)
            == (&other.path, &other.kind, &other.format, other.pretty)
    }
}

impl Ord for Output {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.path, &self.kind, &self.format, self.pretty).cmp(&(
            &other.path,
            &other.kind,
            &other.format,
            other.pretty,
        ))
    }
}

impl PartialOrd for Output {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Output {
    pub fn new(path: &PathBuf, format: OutputFormat) -> Result<Self> {
        debug!("Creating new Output for path: {:?}", path);
        // The only time we don't pretty print is when writing to non-terminal stdout.
        let (writer, kind, pretty): (Box<dyn std::io::Write>, OutputKind, bool) =
            if path.to_str().unwrap_or("<stdout>") == "<stdout>" {
                debug!("Outputting to stdout");
                let is_terminal = std::io::stdout().is_terminal();
                (Box::new(std::io::stdout()), OutputKind::Stdout, is_terminal)
            } else {
                debug!("Outputting to file: {:?}", path);
                let w = get_writer(path)?;
                // Files should always be written with pretty printing for readability.
                (w, OutputKind::new(path), true)
            };

        debug!(
            "Output created with kind: {:?}, format: {:?}, pretty: {}",
            kind, format, pretty
        );
        Ok(Output {
            path: path.clone(),
            writer,
            kind,
            format,
            pretty,
        })
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn writer(&mut self) -> &mut dyn Write {
        &mut self.writer
    }

    pub fn kind(&self) -> &OutputKind {
        &self.kind
    }

    pub fn format(&self) -> &OutputFormat {
        &self.format
    }

    pub fn is_pretty(&self) -> bool {
        self.pretty
    }

    pub fn write<T: serde::Serialize>(&mut self, job: &str, content: &T) -> Result<()> {
        if self.pretty {
            let is_stdout = matches!(self.kind, OutputKind::Stdout);
            pretty(&mut self.writer, content, &self.format, job, is_stdout)
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
    debug!("Writing raw output with format: {:?}", format);
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

    debug!("Writing data:\n{}", data);
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
    job: &str,
    is_stdout: bool,
) -> Result<()> {
    debug!("Writing pretty output with format: {:?}", format);
    let mut data = String::new();
    if is_stdout {
        // Notify user of output file if pretty printing. The only time we don't pretty
        // print is when writing to non-terminal stdout. If it's pretty printing, we assume it's
        // going to another program so we don't want to add extra output.
        data = job.to_string() + ":\n";
    }

    let res = match format {
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

    data += &res;
    if is_stdout {
        data += "\n";
    }

    debug!("Writing data");
    match writer.write_all(data.as_bytes()) {
        Ok(_) => Ok(()),
        Err(e) => Err(Error::new(SourceError::Io(e))
            .context("Failed to write output")
            .code(CODE_RUNTIME_ERROR)),
    }
}
