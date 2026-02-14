use core::ecdsa;
use ibc::lightclient::{ILightClientDispatcher, ILightClientDispatcherTrait};
use ibc::types::{ClientId, ClientIdImpl, TimestampTrait};
use loopback_light_client::types::{ClientState, ConsensusState};
use snforge_std::fs::{FileTrait, read_txt};
use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
use starknet::{ClassHash, ContractAddress, SyscallResultTrait};
use zasset::{IZAsset, IZAssetDispatcher, IZAssetDispatcherTrait};

#[derive(Serde, Drop)]
struct ZAssetConstructorCalldata {
    erc20_name: ByteArray,
    erc20_symbol: ByteArray,
    client_address: ContractAddress,
    zkp_verifier_class_hash: ClassHash,
    attestor_pubkey: felt252,
    token_address: felt252,
    balance_slot: felt252,
}

fn deploy_contract(name: ByteArray, constructor_calldata: @Array<felt252>) -> ContractAddress {
    let contract = declare(name).unwrap_syscall().contract_class();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap_syscall();
    contract_address
}

fn deploy_zasset_and_zkp_verifier() -> (IZAssetDispatcher, ILightClientDispatcher) {
    let starknet_light_client = deploy_contract("StarknetLightClient", @array![]);
    let zkp_verifier_class_hash = *declare("Groth16VerifierBN254")
        .unwrap_syscall()
        .contract_class()
        .class_hash;

    let mut buf = array![];
    Serde::serialize(
        @ZAssetConstructorCalldata {
            erc20_name: "Union",
            erc20_symbol: "U",
            client_address: starknet_light_client,
            zkp_verifier_class_hash,
            attestor_pubkey: 0x54b31cf3f06e130abd82e08b464984d10326d12be15b48a357dbebf21b96ecd,
            token_address: 0x4718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d,
            balance_slot: 0x3a4e8ec16e258a799fe707996fd5d21d42b29adc1499a370edf7f809d8c458a,
        },
        ref buf,
    );

    let zasset = deploy_contract("ZAsset", @buf);

    let zasset_dispatcher = IZAssetDispatcher { contract_address: zasset };

    let client_dispatcher = ILightClientDispatcher { contract_address: starknet_light_client };

    (zasset_dispatcher, client_dispatcher)
}

#[derive(Drop)]
struct TestContext {
    caller: ContractAddress,
    relayer: ContractAddress,
    client_id: ClientId,
    zasset_dispatcher: IZAssetDispatcher,
    client_dispatcher: ILightClientDispatcher,
}

#[generate_trait]
impl TestContextImpl of TestContextTrait {
    fn new() -> TestContext {
        let (zasset_dispatcher, client_dispatcher) = deploy_zasset_and_zkp_verifier();
        TestContext {
            caller: 'caller'.try_into().unwrap(),
            relayer: 'relayer'.try_into().unwrap(),
            client_id: ClientIdImpl::new(1_u32.try_into().unwrap()),
            zasset_dispatcher,
            client_dispatcher,
        }
    }
}

#[test]
#[fork(url: "https://api.zan.top/public/starknet-sepolia/rpc/v0_10", block_number: 5931974)]
fn test_redeem() {
    let ctx = TestContextImpl::new();
    let mut client_state_bytes = array![];
    Serde::serialize(
        @ClientState { chain_id: 'SN_SEPOLIA', latest_height: 5888930 }, ref client_state_bytes,
    );
    let mut consensus_state_bytes = array![];
    Serde::serialize(
        @ConsensusState {
            contracts_trie_root: 0x3564dfc3ce13af28268946a941c4fe54b7b394c4fc8194199205e1f8360a1ba,
            classes_trie_root: 0x35d3fd49f03402af0654620fb53efb8f9f45ad802f02b1cb17609833d061663,
            timestamp: TimestampTrait::from_secs(1769594853),
        },
        ref consensus_state_bytes,
    );
    ctx
        .client_dispatcher
        .create_client(
            ctx.caller, ctx.client_id, client_state_bytes, consensus_state_bytes, ctx.relayer,
        );

    let file = FileTrait::new("tests/proof_calldata.txt");
    let calldata = read_txt(@file).span();
    let nullifier = 20522657995848562181293397227937452315298050645971247591816184472910137891607;
    let redeem_amount = 16;
    let beneficiary = 0.try_into().unwrap();
    let attested_message =
        2855871429298314541839339535059048511721869218505467636158466700564625799666;

    ctx
        .zasset_dispatcher
        .redeem(
            ctx.client_id,
            5888930,
            nullifier,
            redeem_amount,
            beneficiary,
            attested_message,
            (
                0x288e5663db75ed47bf27515d5734d214fe95563ed26f67977c042b399ea890d,
                0x62d90311b9f309917bffbaaf24c863c22c9fdd53730d966805e791c047d160c,
            ),
            false,
            calldata,
        )
        .unwrap();
}
