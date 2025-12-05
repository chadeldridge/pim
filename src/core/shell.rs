use crate::core::error::*;
use crate::core::input::{Input, InputKind};
use crate::core::output::{Output, OutputFormat};
use std::path::PathBuf;

pub struct Shell {
    pub input: Input,
    pub output: Output,
}

impl Shell {
    pub fn new(
        input_path: &PathBuf,
        output_path: &PathBuf,
        output_format: OutputFormat,
    ) -> Result<Self> {
        let input = Input::new(input_path)?;
        //dbg!(&input);
        let output = Output::new(output_path, output_format)?;
        dbg!(&output);
        Ok(Shell { input, output })
    }

    pub fn is_terminal(&self) -> bool {
        self.input.is_terminal
    }

    pub fn input_kind(&self) -> &InputKind {
        &self.input.kind
    }

    pub fn read_input(&mut self) -> Result<String> {
        let content = match self.input.read_content() {
            Ok(_) => self.input.content.clone(),
            Err(e) => return Err(e),
        };
        Ok(content)
    }

    pub fn write_output<T: serde::Serialize>(&mut self, content: &T) -> Result<()> {
        self.output.write(content)
    }
}
