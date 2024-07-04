mod ibc_client;
mod msg;
pub mod storage_utils;

#[doc(hidden)]
pub use dlmalloc::GlobalDlmalloc;
pub use ibc_client::*;
pub use msg::*;

#[macro_export]
macro_rules! define_cosmwasm_light_client_contract {
    ($light_client:ident, $client_type:ident) => {
        #[global_allocator]
        static ALLOC: $crate::GlobalDlmalloc = $crate::GlobalDlmalloc;

        #[entry_point]
        pub fn sudo(
            deps: cosmwasm_std::DepsMut<
                <$light_client as ics008_wasm_client::IbcClient>::CustomQuery,
            >,
            env: cosmwasm_std::Env,
            msg: ics008_wasm_client::SudoMsg,
        ) -> ::std::result::Result<
            cosmwasm_std::Response,
            unionlabs::ErrorReporter<ics008_wasm_client::IbcClientError<$light_client>>,
        > {
            let result = <$light_client as ics008_wasm_client::IbcClient>::sudo(deps, env, msg)
                .map_err(unionlabs::ErrorReporter)?;
            Ok(cosmwasm_std::Response::default().set_data(result))
        }

        #[entry_point]
        pub fn query(
            deps: cosmwasm_std::Deps<<$light_client as ics008_wasm_client::IbcClient>::CustomQuery>,
            env: cosmwasm_std::Env,
            msg: ics008_wasm_client::QueryMsg,
        ) -> ::std::result::Result<
            cosmwasm_std::QueryResponse,
            unionlabs::ErrorReporter<ics008_wasm_client::IbcClientError<$light_client>>,
        > {
            <$light_client as ics008_wasm_client::IbcClient>::query(deps, env, msg)
                .map_err(unionlabs::ErrorReporter)
        }

        unionlabs::export_wasm_client_type!($client_type);
    };
}
