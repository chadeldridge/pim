use crate::core::error::*;
use log::debug;
use std::{
    fs::{File, Metadata},
    io::{BufRead, BufReader, Read, Stdin, Write, stdin},
    path::Path,
};

#[derive(Debug)]
pub enum Reader {
    Stdin(BufReader<Stdin>),
    File(BufReader<File>),
    None,
}

impl Reader {
    pub fn new(path: &Path) -> Result<Self> {
        // Check for stdin first.
        if path_is_stdin(path) {
            debug!("Creating Stdin reader");
            //return Ok(Reader::Stdin(stdin()));
            return Ok(Reader::Stdin(BufReader::new(stdin())));
        }

        // Otherwise, create a file reader.
        debug!("Creating File reader for path: {}", path.display());
        let file = File::open(path).map_err(|e| {
            Error::new(SourceError::Io(e))
                .context(format!("Failed to open input file: {}", path.display()).as_str())
                .code(CODE_RUNTIME_ERROR)
                .print_help()
        })?;
        Ok(Reader::File(BufReader::new(file)))
    }
}

impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Reader::Stdin(stdin) => stdin.read(buf),
            Reader::File(file) => file.read(buf),
            Reader::None => Ok(0),
        }
    }
}

impl BufRead for Reader {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        match self {
            Reader::Stdin(stdin) => stdin.fill_buf(),
            Reader::File(file) => file.fill_buf(),
            Reader::None => Ok(&[]),
        }
    }

    fn consume(&mut self, amt: usize) {
        match self {
            Reader::Stdin(stdin) => stdin.consume(amt),
            Reader::File(file) => file.consume(amt),
            Reader::None => {}
        }
    }
}

#[derive(Debug)]
pub enum Writer {
    Stdout(std::io::Stdout),
    File(std::fs::File),
    None,
}

impl Writer {
    pub fn new(path: &Path) -> Result<Self> {
        // Check for stdout first.
        if path.to_str().unwrap_or("<stdout>") == "<stdout>" {
            debug!("Creating Stdout writer");
            return Ok(Writer::Stdout(std::io::stdout()));
        }

        // Check for directory next.
        if path.is_dir() {
            debug!("Creating None writer for directory: {}", path.display());
            return Ok(Writer::None);
        }

        // Otherwise, create a file writer.
        debug!("Creating File writer for path: {}", path.display());
        let file = File::create(path).map_err(|e| {
            Error::new(SourceError::Io(e))
                .context(format!("Failed to create output file: {}", path.display()).as_str())
                .code(CODE_RUNTIME_ERROR)
        })?;
        Ok(Writer::File(file))
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match self {
            Writer::Stdout(stdout) => stdout.write_all(buf).map_err(|e| {
                Error::new(SourceError::Io(e))
                    .context("Writing to stdout")
                    .code(CODE_RUNTIME_ERROR)
            }),
            Writer::File(file) => file.write_all(buf).map_err(|e| {
                Error::new(SourceError::Io(e))
                    .context("Writing to file")
                    .code(CODE_RUNTIME_ERROR)
            }),
            Writer::None => {
                debug!("No writer available (None), skipping write");
                Ok(())
            }
        }
    }
}

fn path_is_stdin(path: &Path) -> bool {
    path == Path::new("-") || path.to_str().unwrap_or("<stdout>") == "<stdin>"
}

pub fn read_first_line<R: BufRead>(mut reader: R) -> Result<String> {
    let mut content = String::new();
    // read_line returns the number of bytes read, which we do not care about here.
    let _ = reader.read_line(&mut content).map_err(|e| {
        Error::new(SourceError::Io(e))
            .context("reading first line")
            .code(CODE_RUNTIME_ERROR)
    })?;
    Ok(content)
}

pub fn is_dir(metadata: &Option<Metadata>) -> bool {
    match metadata {
        Some(md) => md.is_dir(),
        // The input should only have None if it did not exists, in which case we should
        // have returned an error already, so this should never match.
        None => false,
    }
}
