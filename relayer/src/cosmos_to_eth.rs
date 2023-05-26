use std::sync::Arc;

use contracts::{
    glue::{
        self, GoogleProtobufDurationData, GoogleProtobufTimestampData,
        IbcCoreCommitmentV1MerkleRootData, UnionIbcLightclientsCometblsV1ClientStateData,
        UnionIbcLightclientsCometblsV1ConsensusStateData,
        UnionIbcLightclientsCometblsV1FractionData,
    },
    ibc_handler::MsgCreateClient,
    shared_types::IbcCoreClientV1HeightData,
};
use ethers::abi::AbiEncode;
use ethers::{
    prelude::{k256::ecdsa::SigningKey, MiddlewareBuilder},
    providers::{Http, Provider},
    signers::Wallet,
    types::Address,
};
use hex_literal::hex;
use protos::union::ibc::lightclients::cometbls;
use protos::{
    cosmos::staking::{self, v1beta1::QueryParamsRequest},
    ibc::core::commitment::v1::MerkleRoot,
};
use tendermint_rpc::{
    endpoint::{abci_query, block, commit, consensus_state, header},
    Client, HttpClient,
};

use crate::ETH_RPC_API;

pub async fn update_contract() {
    const IBC_HANDLER_ADDRESS: &str = "0xF8F7758FbcEfd546eAEff7dE24AFf666B6228e73";

    const CLIENT_ADDRESS: &str = "0xB8EA8cB425d85536b158d661da1ef0895Bb92F1D";

    let s = SigningKey::from_slice(&hex!(
        "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
    ))
    .unwrap();

    let client = Arc::new(
        Provider::<Http>::try_from(ETH_RPC_API)
            .unwrap()
            .with_signer(Wallet::from(s)),
    );

    let address: Address = IBC_HANDLER_ADDRESS.parse().unwrap();

    let contract = contracts::ibc_handler::IBCHandler::new(address, client);

    // contract
    //     .register_client("cometbls".to_string(), CLIENT_ADDRESS.parse().unwrap())
    //     .call()
    //     .await
    //     .unwrap();

    let tm_client = HttpClient::new("http://0.0.0.0:26657").unwrap();

    // let consensus_state: consensus_state::Response = tm_client.consensus_state().await.unwrap();
    // let abci_query = tm_client
    //     .abci_query(None, [], Some(1_u32.into()), true)
    //     .await
    //     .unwrap();
    // let header: header::Response = tm_client.header(1_u32).await.unwrap();
    let commit: commit::Response = tm_client.latest_commit().await.unwrap();

    let mut staking_client =
        staking::v1beta1::query_client::QueryClient::connect("http://0.0.0.0:9090")
            .await
            .unwrap();

    let params = staking_client
        .params(staking::v1beta1::QueryParamsRequest {})
        .await
        .unwrap()
        .into_inner()
        .params
        .unwrap();

    let height = commit.signed_header.header.height;

    dbg!(height);

    // let consensus_params = tm_client.consensus_params(height).await.unwrap();

    // dbg!(consensus_state);

    let unbonding_period = std::time::Duration::new(
        params
            .unbonding_time
            .clone()
            .unwrap()
            .seconds
            .try_into()
            .unwrap(),
        params
            .unbonding_time
            .clone()
            .unwrap()
            .nanos
            .try_into()
            .unwrap(),
    );

    contract
        .create_client(MsgCreateClient {
            client_type: "cometbls".to_string(),
            client_state_bytes: UnionIbcLightclientsCometblsV1ClientStateData {
                chain_id: "union-devnet".to_string(),
                // https://github.com/cometbft/cometbft/blob/da0e55604b075bac9e1d5866cb2e62eaae386dd9/light/verifier.go#L16
                trust_level: UnionIbcLightclientsCometblsV1FractionData {
                    numerator: 1,
                    denominator: 3,
                },
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                trusting_period: GoogleProtobufDurationData {
                    seconds: (unbonding_period * 85 / 100).as_secs().try_into().unwrap(),
                    nanos: (unbonding_period * 85 / 100)
                        .subsec_nanos()
                        .try_into()
                        .unwrap(),
                },
                unbonding_period: GoogleProtobufDurationData {
                    seconds: unbonding_period.as_secs().try_into().unwrap(),
                    nanos: unbonding_period.subsec_nanos().try_into().unwrap(),
                },
                // https://github.com/cosmos/relayer/blob/23d1e5c864b35d133cad6a0ef06970a2b1e1b03f/relayer/chains/cosmos/provider.go#L177
                max_clock_drift: GoogleProtobufDurationData {
                    seconds: 60 * 10,
                    nanos: 0,
                },
                frozen_height: IbcCoreClientV1HeightData {
                    revision_number: 0,
                    revision_height: 0,
                },
                latest_height: IbcCoreClientV1HeightData {
                    revision_number: 0,
                    revision_height: height.value(),
                },
            }
            .encode()
            .into(),
            consensus_state_bytes: UnionIbcLightclientsCometblsV1ConsensusStateData {
                timestamp: {
                    let ts = commit.signed_header.header.time;
                    GoogleProtobufTimestampData {
                        secs: ts.unix_timestamp(),
                        nanos: (ts.unix_timestamp_nanos()
                            - (ts.unix_timestamp() as i128 * 1_000_000_000_i128))
                            .try_into()
                            .unwrap(),
                    }
                },
                root: IbcCoreCommitmentV1MerkleRootData {
                    hash: commit
                        .signed_header
                        .header
                        .app_hash
                        .as_bytes()
                        .to_vec()
                        .into(),
                },
                next_validators_hash: commit
                    .signed_header
                    .header
                    .next_validators_hash
                    .as_bytes()
                    .to_vec()
                    .into(),
            }
            .encode()
            .into(),
        })
        .call()
        .await
        .unwrap();
}
