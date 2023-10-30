use tracing::subscriber::with_default;
use tracing::Level;
use tracing_attributes::instrument;
use tracing_mock::*;

// Reproduces a compile error when an instrumented function body contains inner
// attributes (https://github.com/tokio-rs/tracing/issues/2294).
#[deny(unused_variables)]
#[instrument]
fn repro_2294() {
    #![allow(unused_variables)]
    let i = 42;
}

#[test]
fn override_everything() {
    #[instrument(target = "my_target", level = "debug")]
    fn my_fn() {}

    #[instrument(level = "debug", target = "my_target")]
    fn my_other_fn() {}

    let span = span::mock()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");
    let span2 = span::mock()
        .named("my_other_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");
    let (subscriber, handle) = subscriber::mock()
        .new_span(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .new_span(span2.clone())
        .enter(span2.clone())
        .exit(span2.clone())
        .drop_span(span2)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn();
        my_other_fn();
    });

    handle.assert_finished();
}

#[test]
fn fields() {
    #[instrument(target = "my_target", level = "debug")]
    fn my_fn(arg1: usize, arg2: bool) {}

    let span = span::mock()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");

    let span2 = span::mock()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span.clone().with_field(
                field::mock("arg1")
                    .with_value(&2usize)
                    .and(field::mock("arg2").with_value(&false))
                    .only(),
            ),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .new_span(
            span2.clone().with_field(
                field::mock("arg1")
                    .with_value(&3usize)
                    .and(field::mock("arg2").with_value(&true))
                    .only(),
            ),
        )
        .enter(span2.clone())
        .exit(span2.clone())
        .drop_span(span2)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn(2, false);
        my_fn(3, true);
    });

    handle.assert_finished();
}

#[test]
fn skip() {
    struct UnDebug(pub u32);

    #[instrument(target = "my_target", level = "debug", skip(_arg2, _arg3))]
    fn my_fn(arg1: usize, _arg2: UnDebug, _arg3: UnDebug) {}

    #[instrument(target = "my_target", level = "debug", skip_all)]
    fn my_fn2(_arg1: usize, _arg2: UnDebug, _arg3: UnDebug) {}

    let span = span::mock()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");

    let span2 = span::mock()
        .named("my_fn")
        .at_level(Level::DEBUG)
        .with_target("my_target");

    let span3 = span::mock()
        .named("my_fn2")
        .at_level(Level::DEBUG)
        .with_target("my_target");

    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span.clone()
                .with_field(field::mock("arg1").with_value(&2usize).only()),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .new_span(
            span2
                .clone()
                .with_field(field::mock("arg1").with_value(&3usize).only()),
        )
        .enter(span2.clone())
        .exit(span2.clone())
        .drop_span(span2)
        .new_span(span3.clone())
        .enter(span3.clone())
        .exit(span3.clone())
        .drop_span(span3)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn(2, UnDebug(0), UnDebug(1));
        my_fn(3, UnDebug(0), UnDebug(1));
        my_fn2(2, UnDebug(0), UnDebug(1));
    });

    handle.assert_finished();
}

#[test]
fn generics() {
    #[derive(Debug)]
    struct Foo;

    #[instrument]
    fn my_fn<S, T: std::fmt::Debug>(arg1: S, arg2: T)
    where
        S: std::fmt::Debug,
    {
    }

    let span = span::mock().named("my_fn");

    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span.clone().with_field(
                field::mock("arg1")
                    .with_value(&format_args!("Foo"))
                    .and(field::mock("arg2").with_value(&format_args!("false"))),
            ),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        my_fn(Foo, false);
    });

    handle.assert_finished();
}

#[test]
fn methods() {
    #[derive(Debug)]
    struct Foo;

    impl Foo {
        #[instrument]
        fn my_fn(&self, arg1: usize) {}
    }

    let span = span::mock().named("my_fn");

    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span.clone().with_field(
                field::mock("self")
                    .with_value(&format_args!("Foo"))
                    .and(field::mock("arg1").with_value(&42usize)),
            ),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let foo = Foo;
        foo.my_fn(42);
    });

    handle.assert_finished();
}

#[test]
fn impl_trait_return_type() {
    #[instrument]
    fn returns_impl_trait(x: usize) -> impl Iterator<Item = usize> {
        0..x
    }

    let span = span::mock().named("returns_impl_trait");

    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span.clone()
                .with_field(field::mock("x").with_value(&10usize).only()),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        for _ in returns_impl_trait(10) {
            // nop
        }
    });

    handle.assert_finished();
}
