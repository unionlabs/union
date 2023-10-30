/// Installs a panic handler that aborts the contract execution
/// and sends the panic message and location to the host.
///
/// This overrides any previous panic handler. See <https://doc.rust-lang.org/std/panic/fn.set_hook.html>
/// for details.
#[cfg(all(feature = "abort", target_arch = "wasm32"))]
pub fn install_panic_handler() {
    use super::imports::handle_panic;
    std::panic::set_hook(Box::new(|info| {
        // E.g. "panicked at 'oh no (a = 3)', src/contract.rs:51:5"
        let full_message = info.to_string();
        handle_panic(&full_message);
    }));
}
