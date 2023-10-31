// These tests require the thread-local scoped dispatcher, which only works when
// we have a standard library. The behaviour being tested should be the same
// with the standard lib disabled.
#![cfg(feature = "std")]

use std::thread;

use tracing::{
    field::{debug, display},
    subscriber::with_default,
    Level, Span,
};
use tracing_mock::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn handles_to_the_same_span_are_equal() {
    // Create a mock subscriber that will return `true` on calls to
    // `Subscriber::enabled`, so that the spans will be constructed. We
    // won't enter any spans in this test, so the subscriber won't actually
    // expect to see any spans.
    with_default(subscriber::mock().run(), || {
        let foo1 = tracing::span!(Level::TRACE, "foo");
        let foo2 = foo1.clone();
        // Two handles that point to the same span are equal.
        assert_eq!(foo1, foo2);
    });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn handles_to_different_spans_are_not_equal() {
    with_default(subscriber::mock().run(), || {
        // Even though these spans have the same name and fields, they will have
        // differing metadata, since they were created on different lines.
        let foo1 = tracing::span!(Level::TRACE, "foo", bar = 1u64, baz = false);
        let foo2 = tracing::span!(Level::TRACE, "foo", bar = 1u64, baz = false);

        assert_ne!(foo1, foo2);
    });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn handles_to_different_spans_with_the_same_metadata_are_not_equal() {
    // Every time time this function is called, it will return a _new
    // instance_ of a span with the same metadata, name, and fields.
    fn make_span() -> Span {
        tracing::span!(Level::TRACE, "foo", bar = 1u64, baz = false)
    }

    with_default(subscriber::mock().run(), || {
        let foo1 = make_span();
        let foo2 = make_span();

        assert_ne!(foo1, foo2);
        // assert_ne!(foo1.data(), foo2.data());
    });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn spans_always_go_to_the_subscriber_that_tagged_them() {
    let subscriber1 = subscriber::mock()
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run();
    let subscriber2 = subscriber::mock().run();

    let foo = with_default(subscriber1, || {
        let foo = tracing::span!(Level::TRACE, "foo");
        foo.in_scope(|| {});
        foo
    });
    // Even though we enter subscriber 2's context, the subscriber that
    // tagged the span should see the enter/exit.
    with_default(subscriber2, move || foo.in_scope(|| {}));
}

// This gets exempt from testing in wasm because of: `thread::spawn` which is
// not yet possible to do in WASM. There is work going on see:
// <https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html>
//
// But for now since it's not possible we don't need to test for it :)
#[test]
fn spans_always_go_to_the_subscriber_that_tagged_them_even_across_threads() {
    let subscriber1 = subscriber::mock()
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run();
    let foo = with_default(subscriber1, || {
        let foo = tracing::span!(Level::TRACE, "foo");
        foo.in_scope(|| {});
        foo
    });

    // Even though we enter subscriber 2's context, the subscriber that
    // tagged the span should see the enter/exit.
    thread::spawn(move || {
        with_default(subscriber::mock().run(), || {
            foo.in_scope(|| {});
        })
    })
    .join()
    .unwrap();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn dropping_a_span_calls_drop_span() {
    let (subscriber, handle) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo");
        span.in_scope(|| {});
        drop(span);
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn span_closes_after_event() {
    let (subscriber, handle) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .event(event::mock())
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "foo").in_scope(|| {
            tracing::event!(Level::DEBUG, {}, "my tracing::event!");
        });
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn new_span_after_event() {
    let (subscriber, handle) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .event(event::mock())
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .enter(span::mock().named("bar"))
        .exit(span::mock().named("bar"))
        .drop_span(span::mock().named("bar"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "foo").in_scope(|| {
            tracing::event!(Level::DEBUG, {}, "my tracing::event!");
        });
        tracing::span!(Level::TRACE, "bar").in_scope(|| {});
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn event_outside_of_span() {
    let (subscriber, handle) = subscriber::mock()
        .event(event::mock())
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        tracing::debug!("my tracing::event!");
        tracing::span!(Level::TRACE, "foo").in_scope(|| {});
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn cloning_a_span_calls_clone_span() {
    let (subscriber, handle) = subscriber::mock()
        .clone_span(span::mock().named("foo"))
        .run_with_handle();
    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo");
        // Allow the "redundant" `.clone` since it is used to call into the `.clone_span` hook.
        #[allow(clippy::redundant_clone)]
        let _span2 = span.clone();
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn drop_span_when_exiting_dispatchers_context() {
    let (subscriber, handle) = subscriber::mock()
        .clone_span(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .run_with_handle();
    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo");
        let _span2 = span.clone();
        drop(span);
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn clone_and_drop_span_always_go_to_the_subscriber_that_tagged_the_span() {
    let (subscriber1, handle1) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .clone_span(span::mock().named("foo"))
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .run_with_handle();
    let subscriber2 = subscriber::mock().done().run();

    let foo = with_default(subscriber1, || {
        let foo = tracing::span!(Level::TRACE, "foo");
        foo.in_scope(|| {});
        foo
    });
    // Even though we enter subscriber 2's context, the subscriber that
    // tagged the span should see the enter/exit.
    with_default(subscriber2, move || {
        let foo2 = foo.clone();
        foo.in_scope(|| {});
        drop(foo);
        drop(foo2);
    });

    handle1.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn span_closes_when_exited() {
    let (subscriber, handle) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        let foo = tracing::span!(Level::TRACE, "foo");

        foo.in_scope(|| {});

        drop(foo);
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn enter() {
    let (subscriber, handle) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .event(event::mock())
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        let foo = tracing::span!(Level::TRACE, "foo");
        let _enter = foo.enter();
        tracing::debug!("dropping guard...");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn entered() {
    let (subscriber, handle) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .event(event::mock())
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        let _span = tracing::span!(Level::TRACE, "foo").entered();
        tracing::debug!("dropping guard...");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn entered_api() {
    let (subscriber, handle) = subscriber::mock()
        .enter(span::mock().named("foo"))
        .event(event::mock())
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo").entered();
        let _derefs_to_span = span.id();
        tracing::debug!("exiting span...");
        let _: Span = span.exit();
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn moved_field() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("foo").with_field(
                field::mock("bar")
                    .with_value(&display("hello from my span"))
                    .only(),
            ),
        )
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        let from = "my span";
        let span = tracing::span!(
            Level::TRACE,
            "foo",
            bar = display(format!("hello from {}", from))
        );
        span.in_scope(|| {});
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn dotted_field_name() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock()
                .named("foo")
                .with_field(field::mock("fields.bar").with_value(&true).only()),
        )
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "foo", fields.bar = true);
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn borrowed_field() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("foo").with_field(
                field::mock("bar")
                    .with_value(&display("hello from my span"))
                    .only(),
            ),
        )
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let from = "my span";
        let mut message = format!("hello from {}", from);
        let span = tracing::span!(Level::TRACE, "foo", bar = display(&message));
        span.in_scope(|| {
            message.insert_str(10, " inside");
        });
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
// If emitting log instrumentation, this gets moved anyway, breaking the test.
#[cfg(not(feature = "log"))]
fn move_field_out_of_struct() {
    use tracing::field::debug;

    #[derive(Debug)]
    struct Position {
        x: f32,
        y: f32,
    }

    let pos = Position {
        x: 3.234,
        y: -1.223,
    };
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("foo").with_field(
                field::mock("x")
                    .with_value(&debug(3.234))
                    .and(field::mock("y").with_value(&debug(-1.223)))
                    .only(),
            ),
        )
        .new_span(
            span::mock()
                .named("bar")
                .with_field(field::mock("position").with_value(&debug(&pos)).only()),
        )
        .run_with_handle();

    with_default(subscriber, || {
        let pos = Position {
            x: 3.234,
            y: -1.223,
        };
        let foo = tracing::span!(Level::TRACE, "foo", x = debug(pos.x), y = debug(pos.y));
        let bar = tracing::span!(Level::TRACE, "bar", position = debug(pos));
        foo.in_scope(|| {});
        bar.in_scope(|| {});
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn float_values() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("foo").with_field(
                field::mock("x")
                    .with_value(&3.234)
                    .and(field::mock("y").with_value(&-1.223))
                    .only(),
            ),
        )
        .run_with_handle();

    with_default(subscriber, || {
        let foo = tracing::span!(Level::TRACE, "foo", x = 3.234, y = -1.223);
        foo.in_scope(|| {});
    });

    handle.assert_finished();
}

// TODO(#1138): determine a new syntax for uninitialized span fields, and
// re-enable these.
/*
#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn add_field_after_new_span() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock()
                .named("foo")
                .with_field(field::mock("bar").with_value(&5)
                .and(field::mock("baz").with_value).only()),
        )
        .record(
            span::mock().named("foo"),
            field::mock("baz").with_value(&true).only(),
        )
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo", bar = 5, baz = false);
        span.record("baz", &true);
        span.in_scope(|| {})
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn add_fields_only_after_new_span() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo"))
        .record(
            span::mock().named("foo"),
            field::mock("bar").with_value(&5).only(),
        )
        .record(
            span::mock().named("foo"),
            field::mock("baz").with_value(&true).only(),
        )
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo", bar = _, baz = _);
        span.record("bar", &5);
        span.record("baz", &true);
        span.in_scope(|| {})
    });

    handle.assert_finished();
}
*/

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn record_new_value_for_field() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("foo").with_field(
                field::mock("bar")
                    .with_value(&5)
                    .and(field::mock("baz").with_value(&false))
                    .only(),
            ),
        )
        .record(
            span::mock().named("foo"),
            field::mock("baz").with_value(&true).only(),
        )
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo", bar = 5, baz = false);
        span.record("baz", &true);
        span.in_scope(|| {})
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn record_new_values_for_fields() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("foo").with_field(
                field::mock("bar")
                    .with_value(&4)
                    .and(field::mock("baz").with_value(&false))
                    .only(),
            ),
        )
        .record(
            span::mock().named("foo"),
            field::mock("bar").with_value(&5).only(),
        )
        .record(
            span::mock().named("foo"),
            field::mock("baz").with_value(&true).only(),
        )
        .enter(span::mock().named("foo"))
        .exit(span::mock().named("foo"))
        .drop_span(span::mock().named("foo"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let span = tracing::span!(Level::TRACE, "foo", bar = 4, baz = false);
        span.record("bar", &5);
        span.record("baz", &true);
        span.in_scope(|| {})
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn new_span_with_target_and_log_level() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock()
                .named("foo")
                .with_target("app_span")
                .at_level(Level::DEBUG),
        )
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        tracing::span!(target: "app_span", Level::DEBUG, "foo");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn explicit_root_span_is_root() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo").with_explicit_parent(None))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        tracing::span!(parent: None, Level::TRACE, "foo");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn explicit_root_span_is_root_regardless_of_ctx() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo"))
        .enter(span::mock().named("foo"))
        .new_span(span::mock().named("bar").with_explicit_parent(None))
        .exit(span::mock().named("foo"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "foo").in_scope(|| {
            tracing::span!(parent: None, Level::TRACE, "bar");
        })
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn explicit_child() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo"))
        .new_span(span::mock().named("bar").with_explicit_parent(Some("foo")))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let foo = tracing::span!(Level::TRACE, "foo");
        tracing::span!(parent: foo.id(), Level::TRACE, "bar");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn explicit_child_at_levels() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo"))
        .new_span(span::mock().named("a").with_explicit_parent(Some("foo")))
        .new_span(span::mock().named("b").with_explicit_parent(Some("foo")))
        .new_span(span::mock().named("c").with_explicit_parent(Some("foo")))
        .new_span(span::mock().named("d").with_explicit_parent(Some("foo")))
        .new_span(span::mock().named("e").with_explicit_parent(Some("foo")))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let foo = tracing::span!(Level::TRACE, "foo");
        tracing::trace_span!(parent: foo.id(), "a");
        tracing::debug_span!(parent: foo.id(), "b");
        tracing::info_span!(parent: foo.id(), "c");
        tracing::warn_span!(parent: foo.id(), "d");
        tracing::error_span!(parent: foo.id(), "e");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn explicit_child_regardless_of_ctx() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo"))
        .new_span(span::mock().named("bar"))
        .enter(span::mock().named("bar"))
        .new_span(span::mock().named("baz").with_explicit_parent(Some("foo")))
        .exit(span::mock().named("bar"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let foo = tracing::span!(Level::TRACE, "foo");
        tracing::span!(Level::TRACE, "bar")
            .in_scope(|| tracing::span!(parent: foo.id(), Level::TRACE, "baz"))
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn contextual_root() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo").with_contextual_parent(None))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "foo");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn contextual_child() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("foo"))
        .enter(span::mock().named("foo"))
        .new_span(
            span::mock()
                .named("bar")
                .with_contextual_parent(Some("foo")),
        )
        .exit(span::mock().named("foo"))
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "foo").in_scope(|| {
            tracing::span!(Level::TRACE, "bar");
        })
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn display_shorthand() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("my_span").with_field(
                field::mock("my_field")
                    .with_value(&display("hello world"))
                    .only(),
            ),
        )
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "my_span", my_field = %"hello world");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn debug_shorthand() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("my_span").with_field(
                field::mock("my_field")
                    .with_value(&debug("hello world"))
                    .only(),
            ),
        )
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "my_span", my_field = ?"hello world");
    });

    handle.assert_finished();
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
#[test]
fn both_shorthands() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span::mock().named("my_span").with_field(
                field::mock("display_field")
                    .with_value(&display("hello world"))
                    .and(field::mock("debug_field").with_value(&debug("hello world")))
                    .only(),
            ),
        )
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        tracing::span!(Level::TRACE, "my_span", display_field = %"hello world", debug_field = ?"hello world");
    });

    handle.assert_finished();
}
