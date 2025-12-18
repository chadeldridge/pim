use crate::core::error::*;
use crate::core::output::{Output, OutputFormat, OutputKind};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(from = "TargetHelper")]
pub struct TargetGroup {
    #[serde(skip_serializing)]
    job: String,
    labels: BTreeMap<String, String>,
    targets: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TargetHelper {
    labels: BTreeMap<String, String>,
    targets: Vec<String>,
}

impl From<TargetHelper> for TargetGroup {
    fn from(helper: TargetHelper) -> Self {
        debug!("Converting TargetHelper into TargetGroup");
        let TargetHelper { labels, targets } = helper;
        let job = match labels.get("job") {
            Some(job) => job.clone(),
            None => "unknown".to_string(),
        };

        TargetGroup {
            job,
            labels,
            targets,
        }
    }
}

impl TargetGroup {
    pub fn new(job: &str, mut labels: BTreeMap<String, String>, targets: Vec<String>) -> Self {
        debug!("Creating new TargetGroup for job '{}'", job);
        if !labels.contains_key(job) {
            labels.insert("job".to_string(), job.to_string());
        }

        TargetGroup {
            job: job.to_string(),
            labels,
            targets,
        }
    }

    pub fn jobs(&self) -> &String {
        &self.job
    }

    pub fn labels(&self) -> &BTreeMap<String, String> {
        &self.labels
    }

    pub fn targets(&self) -> &Vec<String> {
        &self.targets
    }

    pub fn hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        debug!("Hashing TargetGroup for job '{}'", self.job);
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.job.hash(&mut hasher);

        for (k, v) in &self.labels {
            k.hash(&mut hasher);
            v.hash(&mut hasher);
        }

        hasher.finish()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TargetFile {
    job: String,
    output: Output,
    targets: Vec<TargetGroup>,
}

impl TargetFile {
    pub fn new(job: &str, output: &Output, format: &OutputFormat) -> Result<Self> {
        debug!("Creating new TargetFile for job '{}'", job);
        let output_path = match output.kind() {
            OutputKind::Stdout => PathBuf::from("<stdout>"),
            OutputKind::File(path) => path.to_path_buf(),
            OutputKind::Directory(path) => construct_filebuf(&mut path.to_path_buf(), job, format),
        };
        let output = match Output::new(&output_path, format.clone()) {
            Ok(output) => output,
            Err(e) => {
                return Err(e.context(&format!(
                    "Failed to create target output file for job '{}'",
                    job
                )));
            }
        };

        debug!("Created new TargetFile for job '{}'", job);
        Ok(TargetFile {
            job: job.to_string(),
            output,
            targets: Vec::new(),
        })
    }

    pub fn job(&self) -> &String {
        &self.job
    }

    pub fn output(&self) -> &Output {
        &self.output
    }

    pub fn targets(&self) -> &Vec<TargetGroup> {
        &self.targets
    }

    pub fn add_target(&mut self, target: TargetGroup) {
        debug!("Adding TargetGroup to TargetFile for job '{}'", self.job);
        // Check for TargetGroups with the same job and labels. If we find one, merge the targets.
        for tg in &mut self.targets {
            if tg.hash() == target.hash() {
                for t in &target.targets {
                    if !tg.targets.contains(t) {
                        tg.targets.push(t.clone());
                    }
                }
                return;
            }
        }
        self.targets.push(target);
    }

    pub fn write(&mut self) -> Result<()> {
        debug!("Writing TargetFile for job '{}'", self.job);
        self.output.write(&self.job, &self.targets)
    }
}

fn construct_filebuf(path: &mut PathBuf, job: &str, format: &OutputFormat) -> PathBuf {
    debug!(
        "Constructing output file path for job '{}' with format '{:?}'",
        job, format
    );
    path.push(job.to_owned() + "_targets." + format.extension());
    path.to_path_buf()
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
/// A mapping of job names to their corresponding TargetFile.
pub struct TargetFiles {
    files: BTreeMap<String, TargetFile>,
}

impl TargetFiles {
    pub fn insert(&mut self, job: String, target_file: TargetFile) {
        self.files.insert(job, target_file);
    }

    pub fn has_job(&self, job: &str) -> bool {
        self.files.contains_key(job)
    }

    pub fn target_file_mut(&mut self, job: &str) -> Option<&mut TargetFile> {
        self.files.get_mut(job)
    }

    pub fn write_all(&mut self) -> Result<()> {
        debug!("Writing all TargetFiles");
        for (_job, target_file) in self.files.iter_mut() {
            info!(
                "Writing TargetFile for job '{}' to path '{:?}'",
                target_file.job,
                target_file.output.path()
            );
            target_file.write()?;
        }
        Ok(())
    }
}
