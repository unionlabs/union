use glob::{GlobError, PatternError};
use std::ffi::OsString;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    Cargo(io::Error),
    CargoFail,
    Glob(GlobError),
    Io(io::Error),
    Metadata(serde_json::Error),
    Mismatch,
    Open(PathBuf, io::Error),
    Pattern(PatternError),
    ProjectDir,
    ReadStderr(io::Error),
    RunFailed,
    ShouldNotHaveCompiled,
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    UpdateVar(OsString),
    WriteStderr(io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;

        match self {
            Cargo(e) => write!(f, "failed to execute cargo: {}", e),
            CargoFail => write!(f, "cargo reported an error"),
            Glob(e) => write!(f, "{}", e),
            Io(e) => write!(f, "{}", e),
            Metadata(e) => write!(f, "failed to read cargo metadata: {}", e),
            Mismatch => write!(f, "compiler error does not match expected error"),
            Open(path, e) => write!(f, "{}: {}", path.display(), e),
            Pattern(e) => write!(f, "{}", e),
            ProjectDir => write!(f, "failed to determine name of project dir"),
            ReadStderr(e) => write!(f, "failed to read stderr file: {}", e),
            RunFailed => write!(f, "execution of the test case was unsuccessful"),
            ShouldNotHaveCompiled => {
                write!(f, "expected test case to fail to compile, but it succeeded")
            }
            TomlDe(e) => write!(f, "{}", e),
            TomlSer(e) => write!(f, "{}", e),
            UpdateVar(var) => write!(
                f,
                "unrecognized value of TRYBUILD: {:?}",
                var.to_string_lossy(),
            ),
            WriteStderr(e) => write!(f, "failed to write stderr file: {}", e),
        }
    }
}

impl Error {
    pub fn already_printed(&self) -> bool {
        use self::Error::*;

        match self {
            CargoFail | Mismatch | RunFailed | ShouldNotHaveCompiled => true,
            _ => false,
        }
    }
}

impl From<GlobError> for Error {
    fn from(err: GlobError) -> Self {
        Error::Glob(err)
    }
}

impl From<PatternError> for Error {
    fn from(err: PatternError) -> Self {
        Error::Pattern(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::TomlDe(err)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::TomlSer(err)
    }
}
