use substrate_bn::{FieldError, GroupError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("short buffer")]
    ShortBuffer,
    #[error("invalid infinity encoding")]
    InvalidInfinityEncoding,
    #[error("{0:?}")]
    Field(FieldError),
    #[error("{0:?}")]
    Group(GroupError),
    #[error("invalid compressed coordinate, square root doesn't exist")]
    NoSquareRoot,
}

impl From<FieldError> for Error {
    fn from(value: FieldError) -> Self {
        Error::Field(value)
    }
}

impl From<GroupError> for Error {
    fn from(value: GroupError) -> Self {
        Error::Group(value)
    }
}
