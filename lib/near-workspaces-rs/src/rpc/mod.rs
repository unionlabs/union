pub(crate) mod client;
pub(crate) mod tool;

pub mod patch;
pub mod query;

pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
