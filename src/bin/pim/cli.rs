use clap::{CommandFactory, Parser};
use log::debug;
use pim::core::error::*;
use pim::core::{Input, InputKind, Output};
use std::{fs::read_dir, path::PathBuf};

/// Command line arguments for PIM
/// Handles arguments parsing and terminal I/O.
#[derive(Debug, Parser)]
#[command(
    name = "pim",
    version,
    about = "Convert source format to Prometheus file_sd target data"
)]
pub struct Args {
    /// Input source file path. Can be a file or directory.
    //source: PathBuf,
    #[arg(short, long)]
    source: Option<PathBuf>,
    // TODO: Change to output target file argument
    /// Output target file path. Can be a file or directory.
    #[arg(short, long)]
    target: Option<PathBuf>,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}

#[derive(Debug)]
pub struct Cli {
    args: Args,
}

impl Cli {
    pub fn new() -> Self {
        debug!("Initializing CLI");
        Cli { args: Args::new() }
    }

    pub fn args(&self) -> &Args {
        &self.args
    }

    pub fn source(&self) -> PathBuf {
        match &self.args.source {
            Some(p) => p.to_path_buf(),
            None => PathBuf::from("-"),
        }
    }

    pub fn print_help() {
        let _ = Args::command().print_help();
    }

    pub fn inputs(&mut self) -> Result<Vec<Input>> {
        debug!("Getting input sources: {:?}", self.args.source);
        let inputs = get_sources(&self.source())?;
        if inputs.is_empty() {
            return Err(Error::new(SourceError::InvalidInputSource(
                "No valid input sources found".to_string(),
            ))
            .set_code(CODE_OPTIONS_ERROR));
        }

        // If inputs is stdin then we have some extra work to do.
        //
        // Return an error if the input is a terminal. Interactive terminal input isn't realistic
        // for this tool.
        //
        // If it is stdin and not an interactive terminal, update the source arg to reflect that.
        debug!("Validating input sources");
        if inputs.len() == 1 && matches!(inputs[0].kind(), InputKind::Stdin) {
            // Bail on terminal input.
            if inputs[0].is_terminal() {
                return Err(Error::new(SourceError::Msg(
                    "Refusing to run with terminal input/output".to_string(),
                ))
                .set_code(CODE_OPTIONS_ERROR)
                .print_help());
            }

            debug!("Input is stdin, updating source arg");
            self.args.source = Some(PathBuf::from("<stdin>"));
        }

        debug!("Input sources validated, returning Ok");
        Ok(inputs)
    }

    pub fn output(&self) -> Result<Output> {
        debug!("Getting output destination");
        let output_file = match &self.args.target {
            Some(p) => p,
            None => &PathBuf::from("<stdout>"),
        };

        debug!("Output destination obtained: {:?}", output_file);
        Output::new(output_file, Default::default())
    }
}

fn get_sources(path: &PathBuf) -> Result<Vec<Input>> {
    debug!("Getting sources from path: {:?}", path);
    let mut inputs = Vec::new();
    let input = Input::new(path)?;
    match &input.kind() {
        InputKind::Stdin => {
            debug!("Input is stdin");
            inputs.push(input);
        }
        InputKind::File(path) => {
            if input.is_dir() {
                debug!("Input is a directory");
                let mut dir_inputs = from_dir(path)?;
                debug!("Directory inputs obtained: {:?}", dir_inputs);
                inputs.append(&mut dir_inputs);
                return Ok(inputs);
            }

            if input.is_binary() {
                return Err(Error::new(SourceError::InvalidInputSource(
                    path.display().to_string() + ": " + "Binary input is not supported",
                ))
                .set_code(CODE_OPTIONS_ERROR));
            }

            debug!("Input is a file");
            inputs.push(input);
        }
    }

    debug!("Sources obtained: {:?}", inputs);
    Ok(inputs)
}

fn from_dir(path: &PathBuf) -> Result<Vec<Input>> {
    debug!("Getting inputs from directory: {}", path.display());
    let mut inputs = Vec::new();
    let entries = read_dir(path).map_err(|e| {
        Error::new(SourceError::Io(e))
            .set_context(format!("reading directory: {}", path.display()).as_str())
            .set_code(CODE_RUNTIME_ERROR)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            Error::new(SourceError::Io(e))
                .set_context(format!("reading directory entry in: {}", path.display()).as_str())
                .set_code(CODE_RUNTIME_ERROR)
        })?;
        let file_path = entry.path();
        debug!("Processing directory entry: {}", file_path.display());
        if file_path.is_dir() {
            debug!("Entry is a directory, recursing into it");
            let mut dir_inputs = from_dir(&file_path)?;
            inputs.append(&mut dir_inputs);
            continue;
        }

        debug!("Entry is a file, creating Input");
        let input = Input::new(&file_path)?;
        inputs.push(input);
    }

    debug!("Directory inputs obtained: {:?}", inputs);
    Ok(inputs)
}
