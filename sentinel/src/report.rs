use core::fmt::Display;

pub struct Report {
    pub name: String,
    pub result: anyhow::Result<()>,
}

impl Report {
    pub fn new(name: impl Into<String>, result: anyhow::Result<()>) -> Self {
        let name = name.into();
        Self { name, result }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.result {
            Ok(()) => write!(f, "sentinel {} ... ok", self.name),
            Err(err) => write!(f, "sentinel {} ... FAILED\n {:?}", self.name, err),
        }
    }
}
