pub mod client;
pub mod contract;
pub mod errors;
pub mod verifier;

fn impls_send<T: Send>() {}

const _: () = {
    || {
        impls_send::<
            unionlabs::ibc::lightclients::wasm::client_state::TryFromWasmClientStateError<
                tendermint_light_client_types::client_state::ClientState,
            >,
        >()
    };
};
