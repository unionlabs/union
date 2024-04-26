use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
    time::Duration,
};

use serde::Deserialize;
use tracing::error;

/// `UpgradeInfo` is set by the node periodically when a chain upgrade is required.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct UpgradeInfo {
    /// The name of the upgrade; which operators must match to a binary.
    pub name: String,
    /// The height at which to enact the upgrade.
    pub height: u64,
    /// Additional info regarding the upgrade.
    pub info: Option<String>,
}

/// Directly reads the file when poll is called in a blocking manner.
pub struct FileReader {
    current: Option<UpgradeInfo>,
    path: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum FileReaderError {
    #[error("file not found")]
    FileNotFound,
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl FileReader {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            current: None,
            path: path.into(),
        }
    }

    fn read_upgrade_info(&self) -> Result<UpgradeInfo, FileReaderError> {
        let mut file = File::open(&self.path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => FileReaderError::FileNotFound,
            _ => FileReaderError::Io(err),
        })?;

        // Sometimes the node is still writing the file, thus the path exists but reading fails. So we retry this operation.
        let mut i = 0;
        let info = loop {
            let mut buf = String::new();
            file.read_to_string(&mut buf).map_err(FileReaderError::Io)?;
            match serde_json::from_str(&buf).map_err(FileReaderError::Serde) {
                Ok(info) => break info,
                Err(err) => {
                    if i == 5 {
                        error!(target: "unionvisor", "failed to deserialize {}, contents were: {}", self.path.display(), &buf);
                        return Err(err);
                    }
                    i += 1;
                    std::thread::sleep(Duration::from_millis(300));
                    continue;
                }
            }
        };

        Ok(info)
    }

    pub fn poll(&mut self) -> Result<Option<UpgradeInfo>, FileReaderError> {
        match self.read_upgrade_info() {
            Err(FileReaderError::FileNotFound) => Ok(None),
            Ok(info) => {
                if self.current == Some(info.clone()) {
                    Ok(None)
                } else {
                    self.current = Some(info.clone());
                    Ok(Some(info))
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;
    use crate::testdata;

    #[test]
    fn test_read_upgrade_info() {
        let info = FileReader::new("src/testdata/upgrade-info.json")
            .read_upgrade_info()
            .unwrap();
        assert_eq!(info.height, 123);
        assert_eq!(info.name, "upgrade1");
    }

    #[test]
    #[traced_test]
    fn test_read_upgrade_info_when_created() {
        println!("starting test_read_upgrade_info_when_created");
        let tmp = testdata::temp_dir_with(&["test_create_and_read"]);
        println!("tmp: {tmp:?}");
        let home = tmp.path().join("test_create_and_read");
        println!("home: {home:?}");
        let upgrade_path = home.as_path().join("data/upgrade-info.json");
        println!("upgrade_path: {upgrade_path:?}");
        let reader = FileReader::new(upgrade_path.clone());
        reader.read_upgrade_info().unwrap_err();

        let bin_path = home.join("bins/bin.sh");
        assert!(!upgrade_path.exists());
        std::process::Command::new(bin_path)
            .args(vec![
                "foo".to_owned(),
                "bar".to_owned(),
                "baz".to_owned(),
                home.join("data").to_str().unwrap().to_owned(),
            ])
            .output()
            .unwrap();
        assert!(upgrade_path.exists());
        assert_eq!(
            std::fs::read_to_string(upgrade_path).unwrap(),
            r#"{"name": "upgrade1", "height": 123}"#
        );
        reader.read_upgrade_info().unwrap();
    }
}
