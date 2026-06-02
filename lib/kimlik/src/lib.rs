use cosmwasm_std::{StdError, StdResult};
use depolama::{Bytes, Prefix, Store, ValueCodec};
#[doc(hidden)]
pub use {cosmwasm_std, depolama};

#[macro_export]
macro_rules! set_id {
    ($KIMLIK:literal) => {
        const _: &str = $KIMLIK;

        pub mod kimlik {
            use $crate::{
                cosmwasm_std::{DepsMut, StdError, StdResult},
                depolama::StorageExt,
            };

            static KIMLIK: &str = $KIMLIK;

            #[cfg(target_arch = "wasm32")]
            #[repr(C)]
            struct Id(*const u8, usize);

            #[cfg(target_arch = "wasm32")]
            #[unsafe(no_mangle)]
            extern "C" fn kimlik() -> Id {
                Id(KIMLIK.as_ptr(), KIMLIK.len())
            }

            pub fn check_id(deps: DepsMut) -> StdResult<()> {
                deps.storage
                    .upsert_item::<::kimlik::Kimlik, _>(|maybe_id| match maybe_id {
                        Some(id) => {
                            if id == KIMLIK {
                                Ok(KIMLIK.to_owned())
                            } else {
                                Err(StdError::generic_err(format!(
                                    "invalid kimlik: current id is {id}, but the new id is {KIMLIK}",
                                )))
                            }
                        }
                        None => Ok(KIMLIK.to_owned()),
                    })
                    .map(|_| ())
            }
        }
    };
}

pub enum Kimlik {}

impl Store for Kimlik {
    const PREFIX: Prefix = Prefix::new(b"kimlik");

    type Key = ();
    type Value = String;
}

impl ValueCodec<String> for Kimlik {
    fn encode_value(value: &String) -> Bytes {
        value.as_bytes().into()
    }

    fn decode_value(raw: &Bytes) -> StdResult<String> {
        str::from_utf8(raw.as_ref())
            .map_err(|e| StdError::generic_err(format!("invalid kimlik: {e}")))
            .map(ToOwned::to_owned)
    }
}
