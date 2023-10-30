#![cfg(target_arch = "wasm32")]

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use futures_channel::oneshot;
use js_sys::Promise;
use std::ops::FnMut;
use wasm_bindgen::{prelude::*, JsValue};
use wasm_bindgen_futures::{future_to_promise, spawn_local, JsFuture};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
async fn promise_resolve_is_ok_future() {
    let p = js_sys::Promise::resolve(&JsValue::from(42));
    let x = JsFuture::from(p).await.unwrap();
    assert_eq!(x, 42);
}

#[wasm_bindgen_test]
async fn promise_reject_is_error_future() {
    let p = js_sys::Promise::reject(&JsValue::from(42));
    let e = JsFuture::from(p).await.unwrap_err();
    assert_eq!(e, 42);
}

#[wasm_bindgen_test]
async fn ok_future_is_resolved_promise() {
    let p = future_to_promise(async { Ok(JsValue::from(42)) });
    let x = JsFuture::from(p).await.unwrap();
    assert_eq!(x, 42);
}

#[wasm_bindgen_test]
async fn error_future_is_rejected_promise() {
    let p = future_to_promise(async { Err(JsValue::from(42)) });
    let e = JsFuture::from(p).await.unwrap_err();
    assert_eq!(e, 42);
}

#[wasm_bindgen_test]
fn debug_jsfuture() {
    let p = js_sys::Promise::resolve(&JsValue::from(42));
    let f = JsFuture::from(p);
    assert_eq!(&format!("{:?}", f), "JsFuture { ... }");
}

#[wasm_bindgen]
extern "C" {
    fn setTimeout(c: &Closure<dyn FnMut()>);
}

#[wasm_bindgen_test]
async fn oneshot_works() {
    let (tx, rx) = oneshot::channel::<u32>();
    let mut tx = Some(tx);
    let closure = Closure::wrap(Box::new(move || {
        drop(tx.take().unwrap());
    }) as Box<dyn FnMut()>);
    setTimeout(&closure);
    closure.forget();
    rx.await.unwrap_err();
}

#[wasm_bindgen_test]
async fn spawn_local_runs() {
    let (tx, rx) = oneshot::channel::<u32>();
    spawn_local(async {
        tx.send(42).unwrap();
    });
    assert_eq!(rx.await.unwrap(), 42);
}

#[wasm_bindgen_test]
async fn spawn_local_nested() {
    let (ta, mut ra) = oneshot::channel::<u32>();
    let (ts, rs) = oneshot::channel::<u32>();
    let (tx, rx) = oneshot::channel::<u32>();
    // The order in which the various promises and tasks run is important!
    // We want, on different ticks each, the following things to happen
    // 1. A promise resolves, off of which we can spawn our inbetween assertion
    // 2. The outer task runs, spawns in the inner task, and the inbetween promise, then yields
    // 3. The inbetween promise runs and asserts that the inner task hasn't run
    // 4. The inner task runs
    // This depends crucially on two facts:
    // - JsFuture schedules on ticks independently from tasks
    // - The order of ticks is the same as the code flow
    let promise = Promise::resolve(&JsValue::null());

    spawn_local(async move {
        // Create a closure that runs in between the two ticks and
        // assert that the inner task hasn't run yet
        let inbetween = Closure::wrap(Box::new(move |_| {
            assert_eq!(
                ra.try_recv().unwrap(),
                None,
                "Nested task should not have run yet"
            );
        }) as Box<dyn FnMut(JsValue)>);
        let inbetween = promise.then(&inbetween);
        spawn_local(async {
            ta.send(0xdead).unwrap();
            ts.send(0xbeaf).unwrap();
        });
        JsFuture::from(inbetween).await.unwrap();
        assert_eq!(
            rs.await.unwrap(),
            0xbeaf,
            "Nested task should run eventually"
        );
        tx.send(42).unwrap();
    });

    assert_eq!(rx.await.unwrap(), 42);
}

#[wasm_bindgen_test]
async fn spawn_local_err_no_exception() {
    let (tx, rx) = oneshot::channel::<u32>();
    spawn_local(async {});
    spawn_local(async {
        tx.send(42).unwrap();
    });
    let val = rx.await.unwrap();
    assert_eq!(val, 42);
}

#[wasm_bindgen_test]
async fn can_create_multiple_futures_from_same_promise() {
    let promise = js_sys::Promise::resolve(&JsValue::null());
    let a = JsFuture::from(promise.clone());
    let b = JsFuture::from(promise);

    a.await.unwrap();
    b.await.unwrap();
}

#[cfg(feature = "futures-core-03-stream")]
#[wasm_bindgen_test]
async fn can_use_an_async_iterable_as_stream() {
    use futures_lite::stream::StreamExt;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::stream::JsStream;

    let async_iter = js_sys::Function::new_no_args(
        "return async function*() { 
            yield 42;
            yield 24;
        }()",
    )
    .call0(&JsValue::undefined())
    .unwrap()
    .unchecked_into::<js_sys::AsyncIterator>();

    let mut stream = JsStream::from(async_iter);
    assert_eq!(stream.next().await, Some(Ok(JsValue::from(42))));
    assert_eq!(stream.next().await, Some(Ok(JsValue::from(24))));
    assert_eq!(stream.next().await, None);
}
