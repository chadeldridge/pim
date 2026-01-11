use crate::app::target::{TargetFile, TargetFiles, TargetGroup};
use crate::core::error::*;
use crate::core::input::{Input, InputFormat};
use crate::core::output::{Output, OutputFormat};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Source {
    jobs: Vec<String>,
    labels: BTreeMap<String, String>,
    targets: Vec<String>,
}

impl Source {
    pub fn jobs(&self) -> &Vec<String> {
        &self.jobs
    }

    pub fn jobs_mut(&mut self) -> &mut Vec<String> {
        &mut self.jobs
    }

    pub fn labels(&self) -> &BTreeMap<String, String> {
        &self.labels
    }

    pub fn labels_mut(&mut self) -> &mut BTreeMap<String, String> {
        &mut self.labels
    }

    pub fn targets(&self) -> &Vec<String> {
        &self.targets
    }
    pub fn targets_mut(&mut self) -> &mut Vec<String> {
        &mut self.targets
    }

    pub fn into_targets(
        &self,
        output: &Output,
        format: &OutputFormat,
        target_files: &mut TargetFiles,
    ) -> Result<()> {
        debug!("Converting source into target files");
        if self.jobs.is_empty() {
            return Err(Error::new(SourceError::InvalidInputSource(
                "Source must have at least one job".to_string(),
            ))
            .set_code(CODE_RUNTIME_ERROR));
        }

        debug!("Converting jobs into target groups");
        for job in &self.jobs {
            if job.is_empty() {
                return Err(Error::new(SourceError::InvalidInputSource(
                    "Jobs in source cannot be empty".to_string(),
                ))
                .set_code(CODE_RUNTIME_ERROR));
            }

            debug!("Processing job: {}", job);
            if !target_files.has_job(job) {
                let target_file = TargetFile::new(job, output, format)?;
                target_files.insert(job.clone(), target_file);
            }

            debug!("Adding target group to target file for job: {}", job);
            match target_files.target_file_mut(job) {
                Some(tf) => {
                    tf.add_target(TargetGroup::new(
                        job,
                        self.labels.clone(),
                        self.targets.clone(),
                    ));
                }
                None => {
                    // No need to return an error but we do need to warn the user. that there could
                    // be a problem.
                    warn!("Target file for job '{}' not found", job);
                }
            }
        }

        debug!("Source converted into target files successfully");
        Ok(())
    }
}

#[derive(Debug)]
pub struct SourceFile {
    pub inputs: Vec<Input>,
    pub sources: Vec<Source>,
}

impl SourceFile {
    pub fn new(inputs: Vec<Input>) -> SourceFile {
        SourceFile {
            inputs,
            sources: Vec::new(),
        }
    }

    pub fn add_input(&mut self, input: Input) {
        self.inputs.push(input);
    }

    pub fn add_inputs(&mut self, inputs: Vec<Input>) {
        self.inputs.extend(inputs);
    }

    pub fn read_sources(&mut self) -> Result<()> {
        debug!("Reading sources from inputs");
        for input in &mut self.inputs {
            debug!("Reading source from input: {:?}", input);
            /*
            input.read_content()?;
            let content = &input.content();
            if content.is_empty() {
                warn!("Input {:?} is empty, skipping", input.kind());
                continue;
            }
            */

            let mut src: Vec<Source> = match input.format() {
                InputFormat::Json => {
                    //debug!("Deserializing as JSON: {}", content);
                    //serde_json::from_str(content).map_err(|e| {
                    serde_json::from_reader(input.mut_reader()).map_err(|e| {
                        Error::new(SourceError::SerdeJson(e))
                            .set_context("Failed to deserialize source from JSON")
                            .set_code(CODE_RUNTIME_ERROR)
                    })?
                }
                InputFormat::Yaml => {
                    //debug!("Deserializing as YAML: {}", content);
                    //serde_yaml::from_str(content).map_err(|e| {
                    serde_yaml::from_reader(input.mut_reader()).map_err(|e| {
                        Error::new(SourceError::SerdeYaml(e))
                            .set_context("Failed to deserialize source from YAML")
                            .set_code(CODE_RUNTIME_ERROR)
                    })?
                }
                _ => {
                    return Err(Error::new(SourceError::UnsupportedInputFormat(
                        input.format().as_str().to_string(),
                    ))
                    .set_context("Unsupported input format for source")
                    .set_code(CODE_RUNTIME_ERROR));
                }
            };

            debug!("Source deserialized: {:?}", src);
            self.sources.append(&mut src);
        }

        Ok(())
    }

    pub fn into_targets(
        &self,
        output: &Output,
        format: &OutputFormat,
        target_files: &mut TargetFiles,
    ) -> Result<()> {
        debug!("Converting all sources into target files");
        for source in &self.sources {
            debug!("Converting source: {:?}", source);
            source.into_targets(output, format, target_files)?;
        }

        Ok(())
    }
}
