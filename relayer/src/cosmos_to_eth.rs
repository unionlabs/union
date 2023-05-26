use std::sync::Arc;

use contracts::{
    glue::{
        GoogleProtobufDurationData, GoogleProtobufTimestampData, IbcCoreCommitmentV1MerkleRootData,
        UnionIbcLightclientsCometblsV1ClientStateData,
        UnionIbcLightclientsCometblsV1ConsensusStateData,
        UnionIbcLightclientsCometblsV1FractionData,
    },
    ibc_handler::MsgCreateClient,
    shared_types::IbcCoreClientV1HeightData,
};
use ethers::{
    abi::AbiEncode,
    prelude::SignerMiddleware,
    providers::Middleware,
    signers::{LocalWallet, Signer},
    types::U256,
};
use ethers::{
    providers::{Http, Provider},
    types::Address,
};
use protos::cosmos::staking;
use tendermint_rpc::{endpoint::commit, Client, HttpClient};

use crate::ETH_RPC_API;

pub async fn update_contract() {
    const IBC_HANDLER_ADDRESS: &str = "0x00144a7Ca3f73C0cE2272c19B7db4192F48e9411";

    const CLIENT_ADDRESS: &str = "0x433488cec14C4478e5ff18DDC7E7384Fc416f148";

    let provider = Arc::new({
        let provider = Provider::<Http>::try_from(ETH_RPC_API).unwrap();
        let chain_id = provider.get_chainid().await.unwrap();
        let wallet = "4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"
            .parse::<LocalWallet>()
            .unwrap()
            .with_chain_id(chain_id.as_u64());
        SignerMiddleware::new(provider, wallet)
    });

    let address: Address = IBC_HANDLER_ADDRESS.parse().unwrap();

    let contract = contracts::ibc_handler::IBCHandler::new(address, provider);

    contract
        .register_client("cometbls".to_string(), CLIENT_ADDRESS.parse().unwrap())
        .send()
        .await
        .unwrap()
        .await
        .unwrap();

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

    let client_state_bytes = UnionIbcLightclientsCometblsV1ClientStateData {
        chain_id: "union-devnet-1".to_string(),
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
    .encode();

    let consensus_state_bytes = UnionIbcLightclientsCometblsV1ConsensusStateData {
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
    .encode();

    // The story behind this is too dark to be explained, you must personnaly ask hussein.aitlahcen@gmail.com
    let prefix = U256::from(32).encode();
    let normalized_client_state_bytes = prefix
        .iter()
        .copied()
        .chain(client_state_bytes.iter().copied())
        .collect::<Vec<_>>();
    let normalized_consensus_state_bytes = prefix
        .iter()
        .copied()
        .chain(consensus_state_bytes.iter().copied())
        .collect::<Vec<_>>();

    let msg_create_client = MsgCreateClient {
        client_type: "cometbls".to_string(),
        client_state_bytes: normalized_client_state_bytes.into(),
        consensus_state_bytes: normalized_consensus_state_bytes.into(),
    };

    contract
        .create_client(msg_create_client)
        .send()
        .await
        .unwrap()
        .await
        .unwrap();
}
