use std::{
    ffi::OsString,
    fmt::Display,
    fs, io,
    path::PathBuf,
    process::{Command, Stdio},
};

use color_eyre::owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, field::display as as_display};

/// Bundles should have the following structure on the filesystem:
///
/// ```text
/// bundle
/// ├── genesis.json
/// ├── meta.json
/// ├── unionvisor
/// └── versions
///     ├── v0.8.0
///     │   └── uniond
///     └── v0.9.0
///         └── uniond
/// ```
///
/// Or, more generally:
///
/// ```text
/// bundle
/// ├── genesis.json
/// ├── meta.json
/// └── META.VERSIONS_PATH
///     └── BINARY_VERSION_IDENTIFIER
///         └── META.BINARY_NAME
/// ```
///
/// Bundle meta information is defined in meta.json, which gets deserialized to [`BundleMeta`]
#[derive(Clone, Debug)]
pub struct Bundle {
    /// The path of the bundle
    pub path: PathBuf,
    /// The deserialized meta info from `bundle/meta.json`
    meta: BundleMeta,
}

/// Version paths that have not been validated.
/// The inner [`PathBuf`] is not public as it should not be accessed.
pub struct UnvalidatedVersionPath(PathBuf);

impl UnvalidatedVersionPath {
    /// Constructs a new [`UnvalidatedVersionPath`] based on any `PathBuf`]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        Self(path)
    }
}

/// Version paths that have been validated.
/// This means that the binary at this path produces valid output when called with --help
pub struct ValidVersionPath(pub PathBuf);

#[derive(Debug, Error)]
pub enum ValidateVersionPathError {
    #[error("version was not found in bundle: {0}")]
    NotInBundle(PathBuf, #[source] io::Error),
    #[error("calling uniond --help for this version failed")]
    HelpCallFailed(#[source] io::Error),
    #[error("permission denied when executing version in bundle: {0}")]
    PermissionDenied(PathBuf, #[source] io::Error),
    #[error("other IO error")]
    OtherIO(#[source] io::Error),
}

impl UnvalidatedVersionPath {
    /// Validates a [`UnvalidatedVersionPath`], turning it into a [`ValidVersionPath`] if validation is successful
    pub fn validate(&self) -> Result<ValidVersionPath, ValidateVersionPathError> {
        debug!(
            "testing if binary {} is available by calling --help",
            as_display(self.0.display())
        );
        let mut child = Command::new(&self.0)
            .arg("--help")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .map_err(ValidateVersionPathError::HelpCallFailed)?;
        debug!(target: "unionvisor", "killing test call of {}", as_display(self.0.display()));

        if let Err(err) = child.kill() {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return Err(ValidateVersionPathError::NotInBundle(self.0.clone(), err))
                }
                io::ErrorKind::PermissionDenied => {
                    return Err(ValidateVersionPathError::PermissionDenied(
                        self.0.clone(),
                        err,
                    ))
                }
                _ => return Err(ValidateVersionPathError::OtherIO(err)),
            }
        }

        Ok(ValidVersionPath(self.0.clone()))
    }
}

/// Bundle meta info found in `bundle/meta.json`
#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct BundleMeta {
    /// The name of the binary in `bundle/bins/$VERSION/`
    binary_name: String,
    /// The fallback version directory in `bundle/bins/`
    fallback_version: String,
    /// The directory containing a directory for each version
    versions_directory: PathBuf,
}

// pub enum BinaryAvailability {
//     NotFound,
//     PermissionDenied,
//     Ok,
// }

impl Display for Bundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "BUNDLE INFO")?;
        writeln!(f, "┏━━━━━━━━━━━")?;
        writeln!(f, "┃ bundle: {self:?}")?;
        writeln!(f, "┃ genesis: {:?}", self.genesis_json().into_os_string())?;
        writeln!(f, "┃ versions:")?;

        let versions =
            fs::read_dir(self.versions_path()).expect("can't read contents of versions path");
        for version in versions {
            let version = version
                .expect("can't read version dir entry")
                .path()
                .into_os_string();
            writeln!(f, "┃ - {version:?}")?;
        }
        writeln!(f, "┗━━━━━━━━━━━")
    }
}

impl Bundle {
    /// Constructs a new [`Bundle`] based on a path.
    /// Will read `bundle/meta.json` and error if invalid.
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, NewBundleError> {
        let path: PathBuf = path.into();

        // Read `bundle/meta.json` and deserialize into `BundleMeta`
        let meta = path.join("meta.json");
        let meta = fs::read_to_string(meta)?;
        let meta = serde_json::from_str(&meta)?;

        // Check if bundle contains genesis.json
        let genesis = path.join("genesis.json");
        if !genesis.exists() {
            return Err(NewBundleError::NoGenesisJson);
        }

        let bundle = Bundle { path, meta };

        Ok(bundle)
    }

    /// Obtains the path to the binary within the bundle with version `version`.
    pub fn path_to(&self, version: impl Into<OsString>) -> UnvalidatedVersionPath {
        let version = version.into();
        UnvalidatedVersionPath::new(
            self.versions_path()
                .join(version)
                .join(&self.meta.binary_name),
        )
    }

    /// Provides the full path the the versions directory
    pub fn versions_path(&self) -> PathBuf {
        self.path.join(&self.meta.versions_directory)
    }

    /// Returns a [`PathBuf`] to the Bundle's genesis.json
    pub fn genesis_json(&self) -> PathBuf {
        self.path.join("genesis.json")
    }

    /// Construct the path to the fallback verison, based on the [`BundleMeta`]
    pub fn fallback_path(&self) -> Result<ValidVersionPath, ValidateVersionPathError> {
        let fallback_version = &self.meta.fallback_version.clone();
        self.path_to(fallback_version).validate()
    }
}

#[derive(Debug, Error)]
pub enum NewBundleError {
    #[error("cannot read bundle/meta.json")]
    NoMetaJson(#[from] io::Error),
    #[error("cannot read bundle/genesis.json")]
    NoGenesisJson,
    #[error(
        "cannot deserialize bundle/meta.json, please ensure that it adheres to the bundle scheme."
    )]
    DeserializeMeta(#[from] serde_json::Error),
}
