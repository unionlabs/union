use tracing_mock::*;

use std::convert::Infallible;
use std::{future::Future, pin::Pin, sync::Arc};
use tracing::subscriber::with_default;
use tracing_attributes::instrument;

#[instrument]
async fn test_async_fn(polls: usize) -> Result<(), ()> {
    let future = PollN::new_ok(polls);
    tracing::trace!(awaiting = true);
    future.await
}

// Reproduces a compile error when returning an `impl Trait` from an
// instrumented async fn (see https://github.com/tokio-rs/tracing/issues/1615)
#[allow(dead_code)] // this is just here to test whether it compiles.
#[instrument]
async fn test_ret_impl_trait(n: i32) -> Result<impl Iterator<Item = i32>, ()> {
    let n = n;
    Ok((0..10).filter(move |x| *x < n))
}

// Reproduces a compile error when returning an `impl Trait` from an
// instrumented async fn (see https://github.com/tokio-rs/tracing/issues/1615)
#[allow(dead_code)] // this is just here to test whether it compiles.
#[instrument(err)]
async fn test_ret_impl_trait_err(n: i32) -> Result<impl Iterator<Item = i32>, &'static str> {
    Ok((0..10).filter(move |x| *x < n))
}

#[instrument]
async fn test_async_fn_empty() {}

// Reproduces a compile error when an instrumented function body contains inner
// attributes (https://github.com/tokio-rs/tracing/issues/2294).
#[deny(unused_variables)]
#[instrument]
async fn repro_async_2294() {
    #![allow(unused_variables)]
    let i = 42;
}

// Reproduces https://github.com/tokio-rs/tracing/issues/1613
#[instrument]
// LOAD-BEARING `#[rustfmt::skip]`! This is necessary to reproduce the bug;
// with the rustfmt-generated formatting, the lint will not be triggered!
#[rustfmt::skip]
#[deny(clippy::suspicious_else_formatting)]
async fn repro_1613(var: bool) {
    println!(
        "{}",
        if var { "true" } else { "false" }
    );
}

// Reproduces https://github.com/tokio-rs/tracing/issues/1613
// and https://github.com/rust-lang/rust-clippy/issues/7760
#[instrument]
#[deny(clippy::suspicious_else_formatting)]
async fn repro_1613_2() {
    // hello world
    // else
}

// Reproduces https://github.com/tokio-rs/tracing/issues/1831
#[allow(dead_code)] // this is just here to test whether it compiles.
#[instrument]
#[deny(unused_braces)]
fn repro_1831() -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {})
}

// This replicates the pattern used to implement async trait methods on nightly using the
// `type_alias_impl_trait` feature
#[allow(dead_code)] // this is just here to test whether it compiles.
#[instrument(ret, err)]
#[deny(unused_braces)]
#[allow(clippy::manual_async_fn)]
fn repro_1831_2() -> impl Future<Output = Result<(), Infallible>> {
    async { Ok(()) }
}

#[test]
fn async_fn_only_enters_for_polls() {
    let (subscriber, handle) = subscriber::mock()
        .new_span(span::mock().named("test_async_fn"))
        .enter(span::mock().named("test_async_fn"))
        .event(event::mock().with_fields(field::mock("awaiting").with_value(&true)))
        .exit(span::mock().named("test_async_fn"))
        .enter(span::mock().named("test_async_fn"))
        .exit(span::mock().named("test_async_fn"))
        .drop_span(span::mock().named("test_async_fn"))
        .done()
        .run_with_handle();
    with_default(subscriber, || {
        block_on_future(async { test_async_fn(2).await }).unwrap();
    });
    handle.assert_finished();
}

#[test]
fn async_fn_nested() {
    #[instrument]
    async fn test_async_fns_nested() {
        test_async_fns_nested_other().await
    }

    #[instrument]
    async fn test_async_fns_nested_other() {
        tracing::trace!(nested = true);
    }

    let span = span::mock().named("test_async_fns_nested");
    let span2 = span::mock().named("test_async_fns_nested_other");
    let (subscriber, handle) = subscriber::mock()
        .new_span(span.clone())
        .enter(span.clone())
        .new_span(span2.clone())
        .enter(span2.clone())
        .event(event::mock().with_fields(field::mock("nested").with_value(&true)))
        .exit(span2.clone())
        .drop_span(span2)
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        block_on_future(async { test_async_fns_nested().await });
    });

    handle.assert_finished();
}

#[test]
fn async_fn_with_async_trait() {
    use async_trait::async_trait;

    // test the correctness of the metadata obtained by #[instrument]
    // (function name, functions parameters) when async-trait is used
    #[async_trait]
    pub trait TestA {
        async fn foo(&mut self, v: usize);
    }

    // test nesting of async fns with aync-trait
    #[async_trait]
    pub trait TestB {
        async fn bar(&self);
    }

    // test skip(self) with async-await
    #[async_trait]
    pub trait TestC {
        async fn baz(&self);
    }

    #[derive(Debug)]
    struct TestImpl(usize);

    #[async_trait]
    impl TestA for TestImpl {
        #[instrument]
        async fn foo(&mut self, v: usize) {
            self.baz().await;
            self.0 = v;
            self.bar().await
        }
    }

    #[async_trait]
    impl TestB for TestImpl {
        #[instrument]
        async fn bar(&self) {
            tracing::trace!(val = self.0);
        }
    }

    #[async_trait]
    impl TestC for TestImpl {
        #[instrument(skip(self))]
        async fn baz(&self) {
            tracing::trace!(val = self.0);
        }
    }

    let span = span::mock().named("foo");
    let span2 = span::mock().named("bar");
    let span3 = span::mock().named("baz");
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span.clone()
                .with_field(field::mock("self"))
                .with_field(field::mock("v")),
        )
        .enter(span.clone())
        .new_span(span3.clone())
        .enter(span3.clone())
        .event(event::mock().with_fields(field::mock("val").with_value(&2u64)))
        .exit(span3.clone())
        .drop_span(span3)
        .new_span(span2.clone().with_field(field::mock("self")))
        .enter(span2.clone())
        .event(event::mock().with_fields(field::mock("val").with_value(&5u64)))
        .exit(span2.clone())
        .drop_span(span2)
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        let mut test = TestImpl(2);
        block_on_future(async { test.foo(5).await });
    });

    handle.assert_finished();
}

#[test]
fn async_fn_with_async_trait_and_fields_expressions() {
    use async_trait::async_trait;

    #[async_trait]
    pub trait Test {
        async fn call(&mut self, v: usize);
    }

    #[derive(Clone, Debug)]
    struct TestImpl;

    impl TestImpl {
        fn foo(&self) -> usize {
            42
        }
    }

    #[async_trait]
    impl Test for TestImpl {
        // check that self is correctly handled, even when using async_trait
        #[instrument(fields(val=self.foo(), val2=Self::clone(self).foo(), test=%_v+5))]
        async fn call(&mut self, _v: usize) {}
    }

    let span = span::mock().named("call");
    let (subscriber, handle) = subscriber::mock()
        .new_span(
            span.clone().with_field(
                field::mock("_v")
                    .with_value(&5usize)
                    .and(field::mock("test").with_value(&tracing::field::debug(10)))
                    .and(field::mock("val").with_value(&42u64))
                    .and(field::mock("val2").with_value(&42u64)),
            ),
        )
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        block_on_future(async { TestImpl.call(5).await });
    });

    handle.assert_finished();
}

#[test]
fn async_fn_with_async_trait_and_fields_expressions_with_generic_parameter() {
    use async_trait::async_trait;

    #[async_trait]
    pub trait Test {
        async fn call();
        async fn call_with_self(&self);
        async fn call_with_mut_self(&mut self);
    }

    #[derive(Clone, Debug)]
    struct TestImpl;

    // we also test sync functions that return futures, as they should be handled just like
    // async-trait (>= 0.1.44) functions
    impl TestImpl {
        #[instrument(fields(Self=std::any::type_name::<Self>()))]
        fn sync_fun(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
            let val = self.clone();
            Box::pin(async move {
                let _ = val;
            })
        }
    }

    #[async_trait]
    impl Test for TestImpl {
        // instrumenting this is currently not possible, see https://github.com/tokio-rs/tracing/issues/864#issuecomment-667508801
        //#[instrument(fields(Self=std::any::type_name::<Self>()))]
        async fn call() {}

        #[instrument(fields(Self=std::any::type_name::<Self>()))]
        async fn call_with_self(&self) {
            self.sync_fun().await;
        }

        #[instrument(fields(Self=std::any::type_name::<Self>()))]
        async fn call_with_mut_self(&mut self) {}
    }

    //let span = span::mock().named("call");
    let span2 = span::mock().named("call_with_self");
    let span3 = span::mock().named("call_with_mut_self");
    let span4 = span::mock().named("sync_fun");
    let (subscriber, handle) = subscriber::mock()
        /*.new_span(span.clone()
            .with_field(
                field::mock("Self").with_value(&"TestImpler")))
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)*/
        .new_span(
            span2
                .clone()
                .with_field(field::mock("Self").with_value(&std::any::type_name::<TestImpl>())),
        )
        .enter(span2.clone())
        .new_span(
            span4
                .clone()
                .with_field(field::mock("Self").with_value(&std::any::type_name::<TestImpl>())),
        )
        .enter(span4.clone())
        .exit(span4)
        .exit(span2.clone())
        .drop_span(span2)
        .new_span(
            span3
                .clone()
                .with_field(field::mock("Self").with_value(&std::any::type_name::<TestImpl>())),
        )
        .enter(span3.clone())
        .exit(span3.clone())
        .drop_span(span3)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        block_on_future(async {
            TestImpl::call().await;
            TestImpl.call_with_self().await;
            TestImpl.call_with_mut_self().await
        });
    });

    handle.assert_finished();
}

#[test]
fn out_of_scope_fields() {
    // Reproduces tokio-rs/tracing#1296

    struct Thing {
        metrics: Arc<()>,
    }

    impl Thing {
        #[instrument(skip(self, _req), fields(app_id))]
        fn call(&mut self, _req: ()) -> Pin<Box<dyn Future<Output = Arc<()>> + Send + Sync>> {
            // ...
            let metrics = self.metrics.clone();
            // ...
            Box::pin(async move {
                // ...
                metrics // cannot find value `metrics` in this scope
            })
        }
    }

    let span = span::mock().named("call");
    let (subscriber, handle) = subscriber::mock()
        .new_span(span.clone())
        .enter(span.clone())
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        block_on_future(async {
            let mut my_thing = Thing {
                metrics: Arc::new(()),
            };
            my_thing.call(()).await;
        });
    });

    handle.assert_finished();
}

#[test]
fn manual_impl_future() {
    #[allow(clippy::manual_async_fn)]
    #[instrument]
    fn manual_impl_future() -> impl Future<Output = ()> {
        async {
            tracing::trace!(poll = true);
        }
    }

    let span = span::mock().named("manual_impl_future");
    let poll_event = || event::mock().with_fields(field::mock("poll").with_value(&true));

    let (subscriber, handle) = subscriber::mock()
        // await manual_impl_future
        .new_span(span.clone())
        .enter(span.clone())
        .event(poll_event())
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        block_on_future(async {
            manual_impl_future().await;
        });
    });

    handle.assert_finished();
}

#[test]
fn manual_box_pin() {
    #[instrument]
    fn manual_box_pin() -> Pin<Box<dyn Future<Output = ()>>> {
        Box::pin(async {
            tracing::trace!(poll = true);
        })
    }

    let span = span::mock().named("manual_box_pin");
    let poll_event = || event::mock().with_fields(field::mock("poll").with_value(&true));

    let (subscriber, handle) = subscriber::mock()
        // await manual_box_pin
        .new_span(span.clone())
        .enter(span.clone())
        .event(poll_event())
        .exit(span.clone())
        .drop_span(span)
        .done()
        .run_with_handle();

    with_default(subscriber, || {
        block_on_future(async {
            manual_box_pin().await;
        });
    });

    handle.assert_finished();
}
