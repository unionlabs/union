// ensure_no_std/src/main.rs
#![no_std]
#![allow(unused_imports)]

extern crate alloc;

// Import the crates that we want to check if they are fully no-std compliance

use ethereum_consensus;
use ethereum_light_client_verifier;

#[cfg(feature = "sp-core")]
use sp_core;

#[cfg(feature = "sp-io")]
use sp_io;

#[cfg(feature = "sp-runtime")]
use sp_runtime;

#[cfg(feature = "sp-std")]
use sp_std;

use core::panic::PanicInfo;

/*

This function definition checks for the compliance of no-std in
dependencies by causing a compile error if  this crate is
linked with `std`. When that happens, you should see error messages
such as follows:

```
error[E0152]: found duplicate lang item `panic_impl`
  --> no-std-check/src/lib.rs
   |
12 | fn panic(_info: &PanicInfo) -> ! {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: the lang item is first defined in crate `std` (which `offending-crate` depends on)
```

 */
#[cfg(feature = "panic-handler")]
#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
