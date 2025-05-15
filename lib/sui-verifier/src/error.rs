#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Client(Box<dyn core::error::Error>),
}
