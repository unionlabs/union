extern crate self as core;

use tracing::{enabled, event, span, Level};

#[test]
fn span() {
    span!(Level::DEBUG, "foo");
}

#[test]
fn event() {
    event!(Level::DEBUG, "foo");
}

#[test]
fn enabled() {
    enabled!(Level::DEBUG);
}
