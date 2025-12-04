use clap::{CommandFactory, Parser};
use pim::core::error::*;
use pim::core::{InputKind, Shell};
use serde_json::json;
use std::{io::IsTerminal, path::PathBuf};

/// Command line arguments for PIM
/// Handles arguments parsing and terminal I/O.
#[derive(Parser)]
#[command(
    name = "pim-export",
    version = "1.0",
    about = "Export PIM data",
    arg_required_else_help = true
)]
pub struct Args {
    /// Input file path
    file: PathBuf,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}

pub struct Cli {
    pub args: Args,
    shell: Shell,
}

impl Cli {
    pub fn new() -> Result<Self> {
        let mut args = Args::new();
        let shell = Shell::new(&args.file)?;

        if shell.is_terminal() {
            // Taking terminal input is just silly so we print help and exit.
            return Err(Error::new(SourceError::Msg(
                "Refusing to run with terminal input/output".to_string(),
            ))
            .code(CODE_OPTIONS_ERROR)
            .print_help());
        }

        if matches!(shell.input_kind(), InputKind::Stdin) {
            args.file = PathBuf::from("<stdin>");
        }

        Ok(Cli { args, shell })
    }

    pub fn print_help(&self) {
        let _ = Args::command().print_help();
    }

    pub fn read_input(&mut self) -> Result<String> {
        match self.shell.read_input() {
            Ok(c) => Ok(c),
            Err(e) => Err(e),
        }
    }

    pub fn print(&self, content: &str) -> Result<String> {
        let data = if std::io::stdout().is_terminal() {
            serde_json::to_string_pretty(&json!({
                "input": self.args.file.display().to_string(),
                "content": content,
            }))
        } else {
            Ok(json!({
                "input": self.args.file.display().to_string(),
                "content": content,
            })
            .to_string())
        };

        match data {
            Ok(d) => {
                println!("{}", d);
                Ok(d)
            }
            Err(e) => Err(
                Error::new(SourceError::Msg(format!("Error generating output: {}", e,)))
                    .code(CODE_OPTIONS_ERROR)
                    .context("Error generating output"),
            ),
        }
    }
}
