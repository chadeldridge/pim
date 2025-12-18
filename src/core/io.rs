use crate::core::error::*;
use log::debug;
use std::{
    fs::{File, Metadata},
    io::BufRead,
    io::Write,
    path::Path,
};

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

pub fn get_writer(path: &Path) -> Result<Box<dyn Write>> {
    // Determine if the path is a directory or file. If it is a directory, create an empty or
    // default file buffer since it will not be used. If it is a file, create the file buffer.
    if path.is_dir() {
        debug!("Creating output writer for directory: {}", path.display());
        let file = Vec::new();
        Ok(Box::new(file))
    } else {
        debug!("Creating output writer from file: {}", path.display());
        let file = File::create(path).map_err(|e| {
            Error::new(SourceError::Io(e))
                .context(format!("Failed to create output file: {}", path.display()).as_str())
                .code(CODE_RUNTIME_ERROR)
        })?;
        Ok(Box::new(file))
    }
}
