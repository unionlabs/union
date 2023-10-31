cfg_trace! {
    cfg_rt! {
        use core::{
            pin::Pin,
            task::{Context, Poll},
        };
        use pin_project_lite::pin_project;
        use std::future::Future;
        pub(crate) use tracing::instrument::Instrumented;

        #[inline]
        #[track_caller]
        pub(crate) fn task<F>(task: F, kind: &'static str, name: Option<&str>, id: u64) -> Instrumented<F> {
            use tracing::instrument::Instrument;
            let location = std::panic::Location::caller();
            let span = tracing::trace_span!(
                target: "tokio::task",
                "runtime.spawn",
                %kind,
                task.name = %name.unwrap_or_default(),
                task.id = id,
                loc.file = location.file(),
                loc.line = location.line(),
                loc.col = location.column(),
            );
            task.instrument(span)
        }

        pub(crate) fn async_op<P,F>(inner: P, resource_span: tracing::Span, source: &str, poll_op_name: &'static str, inherits_child_attrs: bool) -> InstrumentedAsyncOp<F>
        where P: FnOnce() -> F {
            resource_span.in_scope(|| {
                let async_op_span = tracing::trace_span!("runtime.resource.async_op", source = source, inherits_child_attrs = inherits_child_attrs);
                let enter = async_op_span.enter();
                let async_op_poll_span = tracing::trace_span!("runtime.resource.async_op.poll");
                let inner = inner();
                drop(enter);
                let tracing_ctx = AsyncOpTracingCtx {
                    async_op_span,
                    async_op_poll_span,
                    resource_span: resource_span.clone(),
                };
                InstrumentedAsyncOp {
                    inner,
                    tracing_ctx,
                    poll_op_name,
                }
            })
        }

        #[derive(Debug, Clone)]
        pub(crate) struct AsyncOpTracingCtx {
            pub(crate) async_op_span: tracing::Span,
            pub(crate) async_op_poll_span: tracing::Span,
            pub(crate) resource_span: tracing::Span,
        }


        pin_project! {
            #[derive(Debug, Clone)]
            pub(crate) struct InstrumentedAsyncOp<F> {
                #[pin]
                pub(crate) inner: F,
                pub(crate) tracing_ctx: AsyncOpTracingCtx,
                pub(crate) poll_op_name: &'static str
            }
        }

        impl<F: Future> Future for InstrumentedAsyncOp<F> {
            type Output = F::Output;

            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = self.project();
                let poll_op_name = &*this.poll_op_name;
                let _res_enter = this.tracing_ctx.resource_span.enter();
                let _async_op_enter = this.tracing_ctx.async_op_span.enter();
                let _async_op_poll_enter = this.tracing_ctx.async_op_poll_span.enter();
                trace_poll_op!(poll_op_name, this.inner.poll(cx))
            }
        }
    }
}
cfg_time! {
    #[track_caller]
    pub(crate) fn caller_location() -> Option<&'static std::panic::Location<'static>> {
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        return Some(std::panic::Location::caller());
        #[cfg(not(all(tokio_unstable, feature = "tracing")))]
        None
    }
}

cfg_not_trace! {
    cfg_rt! {
        #[inline]
        pub(crate) fn task<F>(task: F, _: &'static str, _name: Option<&str>, _: u64) -> F {
            // nop
            task
        }
    }
}
