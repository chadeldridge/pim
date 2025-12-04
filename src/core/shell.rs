use crate::core::error::*;
use crate::core::input::{Input, InputKind};
use std::path::PathBuf;

pub struct Shell {
    input: Input,
}

impl Shell {
    pub fn new(input_path: &PathBuf) -> Result<Self> {
        let input = Input::new(input_path)?;
        dbg!(&input);
        Ok(Shell { input })
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
}
