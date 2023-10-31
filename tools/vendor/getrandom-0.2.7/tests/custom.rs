// Test that a custom handler works on wasm32-unknown-unknown
#![cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    feature = "custom",
    not(feature = "js")
))]

use wasm_bindgen_test::wasm_bindgen_test as test;
#[cfg(feature = "test-in-browser")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicU8, Ordering},
};
use getrandom::{getrandom, register_custom_getrandom, Error};

fn len7_err() -> Error {
    NonZeroU32::new(Error::INTERNAL_START + 7).unwrap().into()
}

fn super_insecure_rng(buf: &mut [u8]) -> Result<(), Error> {
    // Length 7 buffers return a custom error
    if buf.len() == 7 {
        return Err(len7_err());
    }
    // Otherwise, increment an atomic counter
    static COUNTER: AtomicU8 = AtomicU8::new(0);
    for b in buf {
        *b = COUNTER.fetch_add(1, Ordering::Relaxed);
    }
    Ok(())
}

register_custom_getrandom!(super_insecure_rng);

#[test]
fn custom_rng_output() {
    let mut buf = [0u8; 4];
    assert_eq!(getrandom(&mut buf), Ok(()));
    assert_eq!(buf, [0, 1, 2, 3]);
    assert_eq!(getrandom(&mut buf), Ok(()));
    assert_eq!(buf, [4, 5, 6, 7]);
}

#[test]
fn rng_err_output() {
    assert_eq!(getrandom(&mut [0; 7]), Err(len7_err()));
}
