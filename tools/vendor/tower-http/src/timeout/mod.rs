//! Middleware for setting timeouts on requests and responses.

mod body;
mod service;

pub use body::{TimeoutBody, TimeoutError};
pub use service::{
    RequestBodyTimeout, RequestBodyTimeoutLayer, ResponseBodyTimeout, ResponseBodyTimeoutLayer,
    Timeout, TimeoutLayer,
};
