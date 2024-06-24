use ics008_wasm_client::IbcClientError;

use crate::client::NearLightClient;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {}

impl From<Error> for IbcClientError<NearLightClient> {
    fn from(_value: Error) -> Self {
        todo!()
    }
}
