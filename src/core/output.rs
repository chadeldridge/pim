pub enum OutputType {
    Stdout,
    Dst(PathBuf),
}

pub enum OutputFormat {
    Json,
    Yaml,
}

pub trait Output {
    fn raw(&self, content: &str) -> Result<(), anyhow::Error>;
    fn raw(&self, content: &str) -> Result<(), anyhow::Error>;
}
