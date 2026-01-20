use ibc_union_spec::ClientId;

/// Representation of the client state of a proof lens client.
///
/// For a proof lens client A->B->C, where the proof lens is running on A and tracking C, the
/// terminology is as follows:
///
/// - B is L1
/// - C is L2
///
/// where C "settles" on B with the client `self.l2_client_id`, and B "settles" on A with
/// `self.l1_client_id`.
///
/// # Supported Encoding Formats
///
/// This struct implements bincode, json (serde) and ethabi serialization and deserialization.
///
/// ## JSON
///
/// JSON encoding is implemented via serde.
///
/// ## Bincode
///
/// Bincode encoding is implemented via [`bincode`].
///
/// ## Bcs
///
/// Bcs encoding is implemented via serde and [`bcs`].
///
/// [`bcs`]: (https://docs.rs/bcs/latest/bcs/)
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(deny_unknown_fields)
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct ClientState {
    /// L2 chain ID. This is the same as the ID of the chain being tracked by `self.l2_client_id`.
    ///
    /// ("C")
    pub l2_chain_id: String,

    /// L1 client ID. This is the ID of the L1 client running on A that is used to check the L2
    /// inclusion proof against.
    ///
    /// ("B" on "A")
    pub l1_client_id: ClientId,

    /// L2 client ID. This is the ID of the L2 client running on B (L1) tracking the C (L2).
    ///
    /// ("C" on "B")
    pub l2_client_id: ClientId,

    /// L2 latest height
    pub l2_latest_height: u64,

    pub timestamp_offset: u16,
}

// avert your eyes, here be dragons
#[cfg(feature = "ethabi")]
pub mod ethabi {
    use ibc_union_spec::ClientId;
    use unionlabs_encoding::impl_ethabi_via_try_from_into;

    use crate::ClientState;

    alloy_sol_types::sol! {
        struct SolClientState {
            string l2ChainId;
            uint32 l1ClientId;
            uint32 l2ClientId;
            uint64 l2LatestHeight;
            uint16 timestampOffset;
        }
    }

    impl_ethabi_via_try_from_into!(ClientState => SolClientState);

    impl From<ClientState> for SolClientState {
        fn from(value: ClientState) -> Self {
            SolClientState {
                l2ChainId: value.l2_chain_id,
                l1ClientId: value.l1_client_id.raw(),
                l2ClientId: value.l2_client_id.raw(),
                l2LatestHeight: value.l2_latest_height,
                timestampOffset: value.timestamp_offset,
            }
        }
    }

    impl TryFrom<SolClientState> for ClientState {
        type Error = Error;

        fn try_from(value: SolClientState) -> Result<Self, Self::Error> {
            Ok(Self {
                l2_chain_id: value.l2ChainId,
                l1_client_id: ClientId::new(
                    value
                        .l1ClientId
                        .try_into()
                        .map_err(|_| Error::InvalidL1ClientId)?,
                ),
                l2_client_id: ClientId::new(
                    value
                        .l2ClientId
                        .try_into()
                        .map_err(|_| Error::InvalidL2ClientId)?,
                ),
                l2_latest_height: value.l2LatestHeight,
                timestamp_offset: value.timestampOffset,
            })
        }
    }

    #[derive(Debug, Clone, PartialEq, thiserror::Error)]
    pub enum Error {
        #[error("l1 client id must be non-zero")]
        InvalidL1ClientId,
        #[error("l2 client id must be non-zero")]
        InvalidL2ClientId,
    }
}
