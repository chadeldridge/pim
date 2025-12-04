use pim::core::error::*;
mod cli;

fn handler() -> Result<bool> {
    // Get command line arguments.
    let mut shell = cli::Cli::new()?;

    // Read input data.
    let content = match shell.read_input() {
        Ok(c) => c,
        Err(e) => {
            print_help(&shell, &e);
            return Err(e);
        }
    };

    match shell.print(&content) {
        Ok(_) => Ok(true),
        Err(e) => {
            print_help(&shell, &e);
            Err(e)
        }
    }
}

fn print_help(shell: &cli::Cli, error: &Error) {
    if error.print_help {
        let _ = shell.print_help();
    }
}

fn main() {
    match handler() {
        Ok(true) => std::process::exit(0),
        Ok(false) => std::process::exit(1),
        Err(e) => {
            handle_error(&e, &mut std::io::stderr().lock());
            std::process::exit(e.code.unwrap_or(1));
        }
    }
}
