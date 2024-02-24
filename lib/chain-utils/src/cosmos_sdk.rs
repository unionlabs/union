use std::sync::Arc;

use prost::Message;
use sha2::Digest;
use tendermint_rpc::{Client, WebSocketClient};
use unionlabs::{
    cosmos::auth::base_account::BaseAccount, google::protobuf::any::Any, hash::H256,
    ibc::core::client::height::IsHeight, id::ConnectionId, parse_wasm_client_type,
    signer::CosmosSigner, traits::Chain, MaybeRecoverableError, TypeUrl, WasmClientType,
};

use crate::{
    cosmos_sdk::cosmos_sdk_error::{CosmosSdkError, SdkError},
    Pool,
};

pub trait CosmosSdkChain: Chain {
    fn grpc_url(&self) -> String;
    fn fee_denom(&self) -> String;
    fn tm_client(&self) -> &WebSocketClient;
    fn signers(&self) -> &Pool<CosmosSigner>;
    fn checksum_cache(&self) -> &Arc<dashmap::DashMap<H256, WasmClientType>>;
}

#[allow(async_fn_in_trait)]
pub trait CosmosSdkChainExt: CosmosSdkChain {
    async fn client_type_of_checksum(&self, checksum: H256) -> WasmClientType {
        if let Some(ty) = self.checksum_cache().get(&checksum) {
            tracing::debug!(
                checksum = %checksum.to_string_unprefixed(),
                ty = ?*ty,
                "cache hit for checksum"
            );

            return *ty;
        };

        tracing::info!(
            checksum = %checksum.to_string_unprefixed(),
            "cache miss for checksum"
        );

        let bz = protos::ibc::lightclients::wasm::v1::query_client::QueryClient::connect(
            self.grpc_url().clone(),
        )
        .await
        .unwrap()
        .code(protos::ibc::lightclients::wasm::v1::QueryCodeRequest {
            checksum: checksum.to_string_unprefixed(),
        })
        .await
        .unwrap()
        .into_inner()
        .data;

        let ty = parse_wasm_client_type(bz).unwrap().unwrap();

        tracing::info!(
            checksum = %checksum.to_string_unprefixed(),
            ?ty,
            "parsed checksum"
        );

        self.checksum_cache().insert(checksum, ty);

        ty
    }

    async fn checksum_of_client_id(&self, client_id: Self::ClientId) -> H256 {
        let client_state = protos::ibc::core::client::v1::query_client::QueryClient::connect(
            self.grpc_url().clone(),
        )
        .await
        .unwrap()
        .client_state(protos::ibc::core::client::v1::QueryClientStateRequest {
            client_id: client_id.to_string(),
        })
        .await
        .unwrap()
        .into_inner()
        .client_state
        .unwrap();

        // NOTE: We only need the checksum, so we don't need to decode the inner state
        assert!(
            client_state.type_url == protos::ibc::lightclients::wasm::v1::ClientState::TYPE_URL
        );

        protos::ibc::lightclients::wasm::v1::ClientState::decode(&*client_state.value)
            .unwrap()
            .checksum
            .try_into()
            .unwrap()
    }

    async fn client_id_of_connection(&self, connection_id: ConnectionId) -> Self::ClientId {
        protos::ibc::core::connection::v1::query_client::QueryClient::connect(
            self.grpc_url().clone(),
        )
        .await
        .unwrap()
        .connection(protos::ibc::core::connection::v1::QueryConnectionRequest {
            connection_id: connection_id.to_string(),
        })
        .await
        .unwrap()
        .into_inner()
        .connection
        .unwrap()
        .client_id
        .parse()
        .unwrap()
    }

    async fn account_info(&self, account: &str) -> BaseAccount {
        let Any(account) =
            protos::cosmos::auth::v1beta1::query_client::QueryClient::connect(self.grpc_url())
                .await
                .unwrap()
                .account(protos::cosmos::auth::v1beta1::QueryAccountRequest {
                    address: account.to_string(),
                })
                .await
                .unwrap()
                .into_inner()
                .account
                .unwrap()
                .try_into()
                .unwrap();

        account
    }

    async fn broadcast_tx_commit(
        &self,
        signer: CosmosSigner,
        messages: impl IntoIterator<Item = protos::google::protobuf::Any> + Clone,
    ) -> Result<H256, BroadcastTxCommitError> {
        use protos::cosmos::tx;

        let account = self.account_info(&signer.to_string()).await;

        #[allow(deprecated)]
        // TODO: types in unionlabs for these types
        let sign_doc = tx::v1beta1::SignDoc {
            body_bytes: tx::v1beta1::TxBody {
                messages: messages.clone().into_iter().collect(),
                // TODO(benluelo): What do we want to use as our memo?
                memo: String::new(),
                timeout_height: 123_123_123,
                extension_options: vec![],
                non_critical_extension_options: vec![],
            }
            .encode_to_vec(),
            auth_info_bytes: tx::v1beta1::AuthInfo {
                signer_infos: [tx::v1beta1::SignerInfo {
                    public_key: Some(protos::google::protobuf::Any {
                        type_url: "/cosmos.crypto.secp256k1.PubKey".to_string(),
                        value: signer.public_key().encode_to_vec(),
                    }),
                    mode_info: Some(tx::v1beta1::ModeInfo {
                        sum: Some(tx::v1beta1::mode_info::Sum::Single(
                            tx::v1beta1::mode_info::Single {
                                mode: tx::signing::v1beta1::SignMode::Direct.into(),
                            },
                        )),
                    }),
                    sequence: account.sequence,
                }]
                .to_vec(),
                fee: Some(tx::v1beta1::Fee {
                    amount: vec![protos::cosmos::base::v1beta1::Coin {
                        // TODO: This needs to be configurable
                        denom: self.fee_denom(),
                        amount: "1".to_string(),
                    }],
                    gas_limit: 5_000_000_000,
                    payer: String::new(),
                    granter: String::new(),
                }),
                tip: None,
            }
            .encode_to_vec(),
            chain_id: self.chain_id().to_string(),
            account_number: account.account_number,
        };

        let signature = signer
            .try_sign(&sign_doc.encode_to_vec())
            .expect("signing failed")
            .to_vec();

        let tx_raw = tx::v1beta1::TxRaw {
            body_bytes: sign_doc.body_bytes,
            auth_info_bytes: sign_doc.auth_info_bytes,
            signatures: [signature].to_vec(),
        };

        let tx_raw_bytes = tx_raw.encode_to_vec();

        let tx_hash_normalized: H256 = sha2::Sha256::new()
            .chain_update(&tx_raw_bytes)
            .finalize()
            .into();
        let tx_hash = hex::encode_upper(&tx_hash_normalized);

        if self
            .tm_client()
            .tx(tx_hash.parse().unwrap(), false)
            .await
            .is_ok()
        {
            tracing::info!(%tx_hash, "tx already included");
            return Ok(hex::decode(tx_hash).unwrap().try_into().unwrap());
        }

        let response_result = self
            .tm_client()
            .broadcast_tx_sync(tx_raw_bytes.clone())
            .await;

        let response = response_result.unwrap();

        assert_eq!(
            tx_hash,
            response.hash.to_string(),
            "tx hash calculated incorrectly"
        );

        tracing::debug!(%tx_hash);

        tracing::info!(check_tx_code = ?response.code, codespace = %response.codespace, check_tx_log = %response.log);

        if response.code.is_err() {
            let value = cosmos_sdk_error::CosmosSdkError::from_code_and_codespace(
                &response.codespace,
                response.code.value(),
            );

            tracing::error!("cosmos tx failed: {}", value);

            // TODO: Return an error here
            return Ok(tx_hash_normalized);
        };

        let mut target_height = self.query_latest_height().await.unwrap().increment();
        let mut i = 0;
        loop {
            let reached_height = 'l: loop {
                let current_height = self.query_latest_height().await.unwrap();
                if current_height.into_height() >= target_height.into_height() {
                    break 'l current_height;
                }
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            };

            let tx_inclusion = self.tm_client().tx(tx_hash.parse().unwrap(), false).await;

            tracing::debug!(?tx_inclusion);

            match tx_inclusion {
                Ok(_) => break Ok(tx_hash_normalized),
                Err(err) if i > 5 => {
                    tracing::warn!("tx inclusion couldn't be retrieved after {} try", i);
                    break Err(BroadcastTxCommitError::Inclusion(err));
                }
                Err(_) => {
                    target_height = reached_height.increment();
                    i += 1;
                    continue;
                }
            }
        }
    }
}

impl<T: CosmosSdkChain> CosmosSdkChainExt for T {}

#[derive(Debug, Clone, thiserror::Error)]
pub enum BroadcastTxCommitError {
    #[error("tx was not included")]
    Inclusion(#[from] tendermint_rpc::Error),
    #[error("tx failed: {0:?}")]
    Tx(CosmosSdkError),
}

impl MaybeRecoverableError for BroadcastTxCommitError {
    fn is_recoverable(&self) -> bool {
        match self {
            // tx wasn't included, retry unconditionally
            Self::Inclusion(_) => true,
            Self::Tx(code) => matches!(
                code,
                CosmosSdkError::SdkError(SdkError::ErrTxInMempoolCache)
                    | CosmosSdkError::SdkError(SdkError::ErrMempoolIsFull)
                    | CosmosSdkError::SdkError(SdkError::ErrTxTimeoutHeight)
                    | CosmosSdkError::SdkError(SdkError::ErrWrongSequence)
            ),
        }
    }
}

#[allow(non_upper_case_globals)] // TODO: Report this upstream to num_enum
pub mod cosmos_sdk_error {
    macro_rules! cosmos_sdk_errors {
        (
            $(
                #[err(name = $Module:ident, codespace = $codespace:literal)]
                var (
                    $(
                    	$Err:ident = errorsmod.Register($ModuleName_:ident, $code:literal, $msg:literal)
                    )+
                )
            )+
        ) => {
            #[derive(
                Debug,
                Clone,
                PartialEq,
                Eq,
                Hash,
                thiserror::Error,
            )]
            pub enum CosmosSdkError {
                $(
                    #[error(transparent)]
                    $Module(#[from] $Module),
                )+
                #[error("unknown error: {0}: {1}")]
                Unknown(String, u32),
            }

            impl CosmosSdkError {
                pub fn from_code_and_codespace(codespace: &str, code: u32) -> Self {
                    match codespace {
                        $(
                            $codespace => $Module::try_from(code)
                                .map(Into::into)
                                .map_err(|x| x.number),
                        )+
                        _ => return Self::Unknown(codespace.to_string(), code),
                    }
                    .map_or_else(
                        |code| Self::Unknown(codespace.to_string(), code),
                        std::convert::identity,
                    )
                }
            }

            $(
                #[derive(
                    Debug,
                    Copy,
                    Clone,
                    PartialEq,
                    Eq,
                    Hash,
                    num_enum::IntoPrimitive,
                    num_enum::TryFromPrimitive,
                    thiserror::Error,
                )]
                #[repr(u32)]
                pub enum $Module {
                    $(
                        #[error($msg)]
                        $Err = $code,
                    )+
                }
            )+
        }
    }

    cosmos_sdk_errors! {
        #[err(name = HostError, codespace = "host")]
        var (
            ErrInvalidID     = errorsmod.Register(SubModuleName, 2, "invalid identifier")
            ErrInvalidPath   = errorsmod.Register(SubModuleName, 3, "invalid path")
            ErrInvalidPacket = errorsmod.Register(SubModuleName, 4, "invalid packet")
        )

        #[err(name = IbcError, codespace = "ibc")]
        var (
            // ErrInvalidSequence is used the sequence number (nonce) is incorrect
            // for the signature.
            ErrInvalidSequence = errorsmod.Register(codespace, 1, "invalid sequence")

            // ErrUnauthorized is used whenever a request without sufficient
            // authorization is handled.
            ErrUnauthorized = errorsmod.Register(codespace, 2, "unauthorized")

            // ErrInsufficientFunds is used when the account cannot pay requested amount.
            ErrInsufficientFunds = errorsmod.Register(codespace, 3, "insufficient funds")

            // ErrUnknownRequest is used when the request body.
            ErrUnknownRequest = errorsmod.Register(codespace, 4, "unknown request")

            // ErrInvalidAddress is used when an address is found to be invalid.
            ErrInvalidAddress = errorsmod.Register(codespace, 5, "invalid address")

            // ErrInvalidCoins is used when sdk.Coins are invalid.
            ErrInvalidCoins = errorsmod.Register(codespace, 6, "invalid coins")

            // ErrOutOfGas is used when there is not enough gas.
            ErrOutOfGas = errorsmod.Register(codespace, 7, "out of gas")

            // ErrInvalidRequest defines an ABCI typed error where the request contains
            // invalid data.
            ErrInvalidRequest = errorsmod.Register(codespace, 8, "invalid request")

            // ErrInvalidHeight defines an error for an invalid height
            ErrInvalidHeight = errorsmod.Register(codespace, 9, "invalid height")

            // ErrInvalidVersion defines a general error for an invalid version
            ErrInvalidVersion = errorsmod.Register(codespace, 10, "invalid version")

            // ErrInvalidChainID defines an error when the chain-id is invalid.
            ErrInvalidChainID = errorsmod.Register(codespace, 11, "invalid chain-id")

            // ErrInvalidType defines an error an invalid type.
            ErrInvalidType = errorsmod.Register(codespace, 12, "invalid type")

            // ErrPackAny defines an error when packing a protobuf message to Any fails.
            ErrPackAny = errorsmod.Register(codespace, 13, "failed packing protobuf message to Any")

            // ErrUnpackAny defines an error when unpacking a protobuf message from Any fails.
            ErrUnpackAny = errorsmod.Register(codespace, 14, "failed unpacking protobuf message from Any")

            // ErrLogic defines an internal logic error, e.g. an invariant or assertion
            // that is violated. It is a programmer error, not a user-facing error.
            ErrLogic = errorsmod.Register(codespace, 15, "internal logic error")

            // ErrNotFound defines an error when requested entity doesn't exist in the state.
            ErrNotFound = errorsmod.Register(codespace, 16, "not found")
        )

        #[err(name = PortError, codespace = "port")]
        var (
            // cspell:ignore binded
            ErrPortExists   = errorsmod.Register(SubModuleName, 2, "port is already binded")
            ErrPortNotFound = errorsmod.Register(SubModuleName, 3, "port not found")
            ErrInvalidPort  = errorsmod.Register(SubModuleName, 4, "invalid port")
            ErrInvalidRoute = errorsmod.Register(SubModuleName, 5, "route not found")
        )

        #[err(name = SdkError, codespace = "sdk")]
        var (
            // ErrTxDecode is returned if we cannot parse a transaction
            ErrTxDecode = errorsmod.Register(RootCodespace, 2, "tx parse error")

            // ErrInvalidSequence is used the sequence number (nonce) is incorrect
            // for the signature
            ErrInvalidSequence = errorsmod.Register(RootCodespace, 3, "invalid sequence")

            // ErrUnauthorized is used whenever a request without sufficient
            // authorization is handled.
            ErrUnauthorized = errorsmod.Register(RootCodespace, 4, "unauthorized")

            // ErrInsufficientFunds is used when the account cannot pay requested amount.
            ErrInsufficientFunds = errorsmod.Register(RootCodespace, 5, "insufficient funds")

            // ErrUnknownRequest to doc
            ErrUnknownRequest = errorsmod.Register(RootCodespace, 6, "unknown request")

            // ErrInvalidAddress to doc
            ErrInvalidAddress = errorsmod.Register(RootCodespace, 7, "invalid address")

            // ErrInvalidPubKey to doc
            ErrInvalidPubKey = errorsmod.Register(RootCodespace, 8, "invalid pubkey")

            // ErrUnknownAddress to doc
            ErrUnknownAddress = errorsmod.Register(RootCodespace, 9, "unknown address")

            // ErrInvalidCoins to doc
            ErrInvalidCoins = errorsmod.Register(RootCodespace, 10, "invalid coins")

            // ErrOutOfGas to doc
            ErrOutOfGas = errorsmod.Register(RootCodespace, 11, "out of gas")

            // ErrMemoTooLarge to doc
            ErrMemoTooLarge = errorsmod.Register(RootCodespace, 12, "memo too large")

            // ErrInsufficientFee to doc
            ErrInsufficientFee = errorsmod.Register(RootCodespace, 13, "insufficient fee")

            // ErrTooManySignatures to doc
            ErrTooManySignatures = errorsmod.Register(RootCodespace, 14, "maximum number of signatures exceeded")

            // ErrNoSignatures to doc
            ErrNoSignatures = errorsmod.Register(RootCodespace, 15, "no signatures supplied")

            // ErrJSONMarshal defines an ABCI typed JSON marshalling error
            ErrJSONMarshal = errorsmod.Register(RootCodespace, 16, "failed to marshal JSON bytes")

            // ErrJSONUnmarshal defines an ABCI typed JSON unmarshalling error
            ErrJSONUnmarshal = errorsmod.Register(RootCodespace, 17, "failed to unmarshal JSON bytes")

            // ErrInvalidRequest defines an ABCI typed error where the request contains
            // invalid data.
            ErrInvalidRequest = errorsmod.Register(RootCodespace, 18, "invalid request")

            // ErrTxInMempoolCache defines an ABCI typed error where a tx already exists
            // in the mempool.
            ErrTxInMempoolCache = errorsmod.Register(RootCodespace, 19, "tx already in mempool")

            // ErrMempoolIsFull defines an ABCI typed error where the mempool is full.
            ErrMempoolIsFull = errorsmod.Register(RootCodespace, 20, "mempool is full")

            // ErrTxTooLarge defines an ABCI typed error where tx is too large.
            ErrTxTooLarge = errorsmod.Register(RootCodespace, 21, "tx too large")

            // ErrKeyNotFound defines an error when the key doesn't exist
            ErrKeyNotFound = errorsmod.Register(RootCodespace, 22, "key not found")

            // ErrWrongPassword defines an error when the key password is invalid.
            ErrWrongPassword = errorsmod.Register(RootCodespace, 23, "invalid account password")

            // ErrorInvalidSigner defines an error when the tx intended signer does not match the given signer.
            ErrorInvalidSigner = errorsmod.Register(RootCodespace, 24, "tx intended signer does not match the given signer")

            // ErrorInvalidGasAdjustment defines an error for an invalid gas adjustment
            ErrorInvalidGasAdjustment = errorsmod.Register(RootCodespace, 25, "invalid gas adjustment")

            // ErrInvalidHeight defines an error for an invalid height
            ErrInvalidHeight = errorsmod.Register(RootCodespace, 26, "invalid height")

            // ErrInvalidVersion defines a general error for an invalid version
            ErrInvalidVersion = errorsmod.Register(RootCodespace, 27, "invalid version")

            // ErrInvalidChainID defines an error when the chain-id is invalid.
            ErrInvalidChainID = errorsmod.Register(RootCodespace, 28, "invalid chain-id")

            // ErrInvalidType defines an error an invalid type.
            ErrInvalidType = errorsmod.Register(RootCodespace, 29, "invalid type")

            // ErrTxTimeoutHeight defines an error for when a tx is rejected out due to an
            // explicitly set timeout height.
            ErrTxTimeoutHeight = errorsmod.Register(RootCodespace, 30, "tx timeout height")

            // ErrUnknownExtensionOptions defines an error for unknown extension options.
            ErrUnknownExtensionOptions = errorsmod.Register(RootCodespace, 31, "unknown extension options")

            // ErrWrongSequence defines an error where the account sequence defined in
            // the signer info doesn't match the account's actual sequence number.
            ErrWrongSequence = errorsmod.Register(RootCodespace, 32, "incorrect account sequence")

            // ErrPackAny defines an error when packing a protobuf message to Any fails.
            ErrPackAny = errorsmod.Register(RootCodespace, 33, "failed packing protobuf message to Any")

            // ErrUnpackAny defines an error when unpacking a protobuf message from Any fails.
            ErrUnpackAny = errorsmod.Register(RootCodespace, 34, "failed unpacking protobuf message from Any")

            // ErrLogic defines an internal logic error, e.g. an invariant or assertion
            // that is violated. It is a programmer error, not a user-facing error.
            ErrLogic = errorsmod.Register(RootCodespace, 35, "internal logic error")

            // ErrConflict defines a conflict error, e.g. when two goroutines try to access
            // the same resource and one of them fails.
            ErrConflict = errorsmod.Register(RootCodespace, 36, "conflict")

            // ErrNotSupported is returned when we call a branch of a code which is currently not
            // supported.
            ErrNotSupported = errorsmod.Register(RootCodespace, 37, "feature not supported")

            // ErrNotFound defines an error when requested entity doesn't exist in the state.
            ErrNotFound = errorsmod.Register(RootCodespace, 38, "not found")

            // ErrIO should be used to wrap internal errors caused by external operation.
            // Examples: not DB domain error, file writing etc...
            ErrIO = errorsmod.Register(RootCodespace, 39, "Internal IO error")

            // ErrAppConfig defines an error occurred if min-gas-prices field in BaseConfig is empty.
            ErrAppConfig = errorsmod.Register(RootCodespace, 40, "error in app.toml")

            // ErrInvalidGasLimit defines an error when an invalid GasWanted value is
            // supplied.
            ErrInvalidGasLimit = errorsmod.Register(RootCodespace, 41, "invalid gas limit")

            // ErrPanic should only be set when we recovering from a panic
            // TODO: Figure out what this is and where it comes from
            // ErrPanic = errorsmod.ErrPanic
        )

        #[err(name = ChannelError, codespace = "channel")]
        var (
            ErrChannelExists             = errorsmod.Register(SubModuleName, 2, "channel already exists")
            ErrChannelNotFound           = errorsmod.Register(SubModuleName, 3, "channel not found")
            ErrInvalidChannel            = errorsmod.Register(SubModuleName, 4, "invalid channel")
            ErrInvalidChannelState       = errorsmod.Register(SubModuleName, 5, "invalid channel state")
            ErrInvalidChannelOrdering    = errorsmod.Register(SubModuleName, 6, "invalid channel ordering")
            ErrInvalidCounterparty       = errorsmod.Register(SubModuleName, 7, "invalid counterparty channel")
            ErrInvalidChannelCapability  = errorsmod.Register(SubModuleName, 8, "invalid channel capability")
            ErrChannelCapabilityNotFound = errorsmod.Register(SubModuleName, 9, "channel capability not found")
            ErrSequenceSendNotFound      = errorsmod.Register(SubModuleName, 10, "sequence send not found")
            ErrSequenceReceiveNotFound   = errorsmod.Register(SubModuleName, 11, "sequence receive not found")
            ErrSequenceAckNotFound       = errorsmod.Register(SubModuleName, 12, "sequence acknowledgement not found")
            ErrInvalidPacket             = errorsmod.Register(SubModuleName, 13, "invalid packet")
            ErrPacketTimeout             = errorsmod.Register(SubModuleName, 14, "packet timeout")
            ErrTooManyConnectionHops     = errorsmod.Register(SubModuleName, 15, "too many connection hops")
            ErrInvalidAcknowledgement    = errorsmod.Register(SubModuleName, 16, "invalid acknowledgement")
            ErrAcknowledgementExists     = errorsmod.Register(SubModuleName, 17, "acknowledgement for packet already exists")
            ErrInvalidChannelIdentifier  = errorsmod.Register(SubModuleName, 18, "invalid channel identifier")

            // packets already relayed errors
            ErrPacketReceived           = errorsmod.Register(SubModuleName, 19, "packet already received")
            ErrPacketCommitmentNotFound = errorsmod.Register(SubModuleName, 20, "packet commitment not found") // may occur for already received acknowledgements or timeouts and in rare cases for packets never sent

            // ORDERED channel error
            ErrPacketSequenceOutOfOrder = errorsmod.Register(SubModuleName, 21, "packet sequence is out of order")

            // cspell:ignore Antehandler
            // Antehandler error
            ErrRedundantTx = errorsmod.Register(SubModuleName, 22, "packet messages are redundant")

            // Perform a no-op on the current Msg
            ErrNoOpMsg = errorsmod.Register(SubModuleName, 23, "message is redundant, no-op will be performed")

            ErrInvalidChannelVersion = errorsmod.Register(SubModuleName, 24, "invalid channel version")
            ErrPacketNotSent         = errorsmod.Register(SubModuleName, 25, "packet has not been sent")
            ErrInvalidTimeout        = errorsmod.Register(SubModuleName, 26, "invalid packet timeout")
        )
    }
}
