use tracing::{subscriber::with_default, Id, Level};
use tracing_attributes::instrument;
use tracing_mock::*;

#[instrument]
fn with_default_parent() {}

#[instrument(parent = parent_span, skip(parent_span))]
fn with_explicit_parent<P>(parent_span: P)
where
    P: Into<Option<Id>>,
{
}

#[test]
fn default_parent_test() {
    let contextual_parent = span::mock().named("contextual_parent");
    let child = span::mock().named("with_default_parent");

    let (subscriber, handle) = subscriber::mock()
        .new_span(
            contextual_parent
                .clone()
                .with_contextual_parent(None)
                .with_explicit_parent(None),
        )
        .new_span(
            child
                .clone()
                .with_contextual_parent(Some("contextual_parent"))
                .with_explicit_parent(None),
        )
        .enter(child.clone())
        .exit(child.clone())
        .enter(contextual_parent.clone())
        .new_span(
            child
                .clone()
                .with_contextual_parent(Some("contextual_parent"))
                .with_explicit_parent(None),
        )
        .enter(child.clone())
        .exit(child)
        .exit(contextual_parent)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let contextual_parent = tracing::span!(Level::TRACE, "contextual_parent");

        with_default_parent();

        contextual_parent.in_scope(|| {
            with_default_parent();
        });
    });

    handle.assert_finished();
}

#[test]
fn explicit_parent_test() {
    let contextual_parent = span::mock().named("contextual_parent");
    let explicit_parent = span::mock().named("explicit_parent");
    let child = span::mock().named("with_explicit_parent");

    let (subscriber, handle) = subscriber::mock()
        .new_span(
            contextual_parent
                .clone()
                .with_contextual_parent(None)
                .with_explicit_parent(None),
        )
        .new_span(
            explicit_parent
                .with_contextual_parent(None)
                .with_explicit_parent(None),
        )
        .enter(contextual_parent.clone())
        .new_span(
            child
                .clone()
                .with_contextual_parent(Some("contextual_parent"))
                .with_explicit_parent(Some("explicit_parent")),
        )
        .enter(child.clone())
        .exit(child)
        .exit(contextual_parent)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let contextual_parent = tracing::span!(Level::INFO, "contextual_parent");
        let explicit_parent = tracing::span!(Level::INFO, "explicit_parent");

        contextual_parent.in_scope(|| {
            with_explicit_parent(&explicit_parent);
        });
    });

    handle.assert_finished();
}
