use env_logger::Env;
use log::{debug, info};
use pim::app::source::SourceFile;
use pim::app::target::TargetFiles;
use pim::core::{Input, Output, error::*};

mod cli;

fn main() {
    // Initialize logger.
    setup_logger();
    debug!("Logger initialized");

    // Get command line arguments or exit.
    let mut shell = cli::Cli::new();
    debug!("Command line arguments parsed: {:?}", shell.args());

    // Run main handler or exit on error.
    debug!("Running main handler\n");
    handler(&mut shell).map_err(|e| exit_handler(&e));

    // Exit successfully.
    std::process::exit(0);
}

fn setup_logger() {
    let env = Env::default()
        .filter_or("PIM_LOG_LEVEL", "error")
        .write_style_or("PIM_LOG_STYLE", "always");
    env_logger::Builder::from_env(env).init();
}

/// Handle error and exit program.
fn exit_handler(error: &Error) -> ! {
    handle_error(error);
    if error.is_print_help() {
        cli::Cli::print_help();
    }
    debug!("Exiting with code {:?}", error.code());
    std::process::exit(error.code().unwrap_or(1));
}

/// Main program handler. Gets inputs and outputs, then run subcommands.
fn handler(shell: &mut cli::Cli) -> Result<()> {
    // Get our inputs and outputs.
    debug!("Getting inputs");
    let inputs = shell.inputs()?;
    debug!("Inputs obtained: {:?}", inputs);
    debug!("Getting outputs");
    let output = shell.output()?;
    debug!("Outputs obtained: {:?}", output);

    // TODO: This will later become a match on a subcommand arguement as new features are added.
    // Run exporter.
    exporter(inputs, output)
}

/// Exporter function to read inputs and write outputs for prometheus file_sd target files.
fn exporter(inputs: Vec<Input>, output: Output) -> Result<()> {
    debug!("Starting export process");
    // Read input data.
    info!("Reading source inputs");
    let mut source = SourceFile::new(inputs);
    debug!("SourceFile initialized: {:?}", source);
    source.read_sources()?;
    debug!("Source inputs read: {:?}\n", source);

    // Write output data.
    // Target files holds the generated targets split into the individual files to be written to.
    info!("Preparing target files for output");
    let mut target_files = TargetFiles::default();
    source.into_targets(&output, output.format(), &mut target_files)?;
    debug!("Target files prepared: {:?}", target_files);
    target_files.write_all()?;

    Ok(())
}
