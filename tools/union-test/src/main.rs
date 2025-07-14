use std::{str::FromStr, time::Duration};

use alloy::{sol_types::SolValue, sol_types::SolCall};
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{FeemarketConfig, GasFillerConfig};
use hex_literal::hex;
use ibc_union_spec::ChannelId;
use std::num::NonZero;
use protos::{cosmos::base::v1beta1::Coin};
use rand::RngCore;
use ucs03_zkgm::{
    self,
    com::{
        FungibleAssetMetadata, FungibleAssetOrderV2, Instruction, Stake, FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE, INSTR_VERSION_0, INSTR_VERSION_2, OP_FUNGIBLE_ASSET_ORDER, OP_STAKE
    },
};

use union_test::{
    channel_provider::ChannelPair, cosmos, evm::{
        self,
        zkgm::{
            Instruction as InstructionEvm,
            UCS03Zkgm::{self}
        },
        zkgmerc20::ZkgmERC20,

    }, TestContext
};
use unionlabs::{
    bech32::Bech32,
    encoding::{Encode, Json},
    primitives::{FixedBytes, U256},
    ethereum::keccak256
};
use ethers::utils::hex;
use voyager_sdk::{anyhow, primitives::ChainId};

// async fn deploy_basic_erc20(module: &evm::Module<'_>) -> Result<H160> {
//     // 1) Your compiled bytecode (creation code) as hex
//     const BYTECODE: &str = "0x608060405234801561000f575f5ffd5b50604051610a7e380380610a7e83398101604081905261002e916101ff565b6040518060400160405280600481526020016311dbdb1960e21b8152506040518060400160405280600381526020016211d31160ea1b815250816003908161007691906102ae565b50600461008382826102ae565b505050610096338261009c60201b60201c565b5061038d565b6001600160a01b0382166100ca5760405163ec442f0560e01b81525f60048201526024015b60405180910390fd5b6100d55f83836100d9565b5050565b6001600160a01b038316610103578060025f8282546100f89190610368565b909155506101739050565b6001600160a01b0383165f90815260208190526040902054818110156101555760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016100c1565b6001600160a01b0384165f9081526020819052604090209082900390555b6001600160a01b03821661018f576002805482900390556101ad565b6001600160a01b0382165f9081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef836040516101f291815260200190565b60405180910390a3505050565b5f6020828403121561020f575f5ffd5b5051919050565b634e487b7160e01b5f52604160045260245ffd5b600181811c9082168061023e57607f821691505b60208210810361025c57634e487b7160e01b5f52602260045260245ffd5b50919050565b601f8211156102a957805f5260205f20601f840160051c810160208510156102875750805b601f840160051c820191505b818110156102a6575f8155600101610293565b50505b505050565b81516001600160401b038111156102c7576102c7610216565b6102db816102d5845461022a565b84610262565b6020601f82116001811461030d575f83156102f65750848201515b5f19600385901b1c1916600184901b1784556102a6565b5f84815260208120601f198516915b8281101561033c578785015182556020948501946001909201910161031c565b508482101561035957868401515f19600387901b60f8161c191681555b50505050600190811b01905550565b8082018082111561038757634e487b7160e01b5f52601160045260245ffd5b92915050565b6106e48061039a5f395ff3fe608060405234801561000f575f5ffd5b5060043610610090575f3560e01c8063313ce56711610063578063313ce567146100fa57806370a082311461010957806395d89b4114610131578063a9059cbb14610139578063dd62ed3e1461014c575f5ffd5b806306fdde0314610094578063095ea7b3146100b257806318160ddd146100d557806323b872dd146100e7575b5f5ffd5b61009c610184565b6040516100a99190610554565b60405180910390f35b6100c56100c03660046105a4565b610214565b60405190151581526020016100a9565b6002545b6040519081526020016100a9565b6100c56100f53660046105cc565b61022d565b604051601281526020016100a9565b6100d9610117366004610606565b6001600160a01b03165f9081526020819052604090205490565b61009c610250565b6100c56101473660046105a4565b61025f565b6100d961015a366004610626565b6001600160a01b039182165f90815260016020908152604080832093909416825291909152205490565b60606003805461019390610657565b80601f01602080910402602001604051908101604052809291908181526020018280546101bf90610657565b801561020a5780601f106101e15761010080835404028352916020019161020a565b820191905f5260205f20905b8154815290600101906020018083116101ed57829003601f168201915b5050505050905090565b5f3361022181858561026c565b60019150505b92915050565b5f3361023a85828561027e565b6102458585856102ff565b506001949350505050565b60606004805461019390610657565b5f336102218185856102ff565b610279838383600161035c565b505050565b6001600160a01b038381165f908152600160209081526040808320938616835292905220545f198110156102f957818110156102eb57604051637dc7a0d960e11b81526001600160a01b038416600482015260248101829052604481018390526064015b60405180910390fd5b6102f984848484035f61035c565b50505050565b6001600160a01b03831661032857604051634b637e8f60e11b81525f60048201526024016102e2565b6001600160a01b0382166103515760405163ec442f0560e01b81525f60048201526024016102e2565b61027983838361042e565b6001600160a01b0384166103855760405163e602df0560e01b81525f60048201526024016102e2565b6001600160a01b0383166103ae57604051634a1406b160e11b81525f60048201526024016102e2565b6001600160a01b038085165f90815260016020908152604080832093871683529290522082905580156102f957826001600160a01b0316846001600160a01b03167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b9258460405161042091815260200190565b60405180910390a350505050565b6001600160a01b038316610458578060025f82825461044d919061068f565b909155506104c89050565b6001600160a01b0383165f90815260208190526040902054818110156104aa5760405163391434e360e21b81526001600160a01b038516600482015260248101829052604481018390526064016102e2565b6001600160a01b0384165f9081526020819052604090209082900390555b6001600160a01b0382166104e457600280548290039055610502565b6001600160a01b0382165f9081526020819052604090208054820190555b816001600160a01b0316836001600160a01b03167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef8360405161054791815260200190565b60405180910390a3505050565b602081525f82518060208401528060208501604085015e5f604082850101526040601f19601f83011684010191505092915050565b80356001600160a01b038116811461059f575f5ffd5b919050565b5f5f604083850312156105b5575f5ffd5b6105be83610589565b946020939093013593505050565b5f5f5f606084860312156105de575f5ffd5b6105e784610589565b92506105f560208501610589565b929592945050506040919091013590565b5f60208284031215610616575f5ffd5b61061f82610589565b9392505050565b5f5f60408385031215610637575f5ffd5b61064083610589565b915061064e60208401610589565b90509250929050565b600181811c9082168061066b57607f821691505b60208210810361068957634e487b7160e01b5f52602260045260245ffd5b50919050565b8082018082111561022757634e487b7160e01b5f52601160045260245ffdfea264697066735822122013e2b3649190ca5e7d98785f56d95c8a740fcd18345dbd406e025315903e1eba64736f6c634300081e0033"; 
//     let mut code = hex::decode(BYTECODE.trim_start_matches("0x"))?;
//     let initial_supply = ethers::types::U256::from_dec_str("1000000000000000000")?;
//     let encoded = abi::encode(&[
//        Token::Uint(initial_supply)
//    ]);

//    code.extend(&encoded);
//     // 2) Get a signed provider
//     let provider = module.get_provider().await;

//     // 3) figure out your from-address (alice) and its pending nonce
//     let from = module
//         .keyring
//         .with(|w| async move { w.address() })
//         .await
//         .unwrap();
//     let nonce = provider
//         .get_transaction_count(from.into())
//         .await?;

//     // 3) Build a *deploy* call
//     let mut call = RawCallBuilder::new_raw_deploy(
//         provider.clone(),     // your DynProvider<AnyNetwork>
//         code.into(),          // the bytecode
//     ).nonce(nonce);;

//     println!("[deploy_basic_erc20] before gas. Nonce: {}", nonce);
//     // 4) Estimate gas + buffer
//     let gas_est = call.estimate_gas().await?;
//     call = call.gas(((gas_est as f64 * 2.2) as u64));
//     println!("[deploy_basic_erc20] Estimated gas: {}", gas_est);
//     // 5) Send & await receipt
//     let pending = call.send().await?;
//     println!("[deploy_basic_erc20] pending: {:?}", pending);
//     let receipt = pending.get_receipt().await?;
//     println!("[deploy_basic_erc20] receipt: {:?}", receipt);

//     // 6) Extract the new contract address
//     let address = receipt
//         .contract_address
//         .expect("deploy didn’t return an address");
//     Ok(address.into())
// }

// use evm::zkgm::{UCS03Zkgm};
#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let quote_token_addr = "756e696f6e3174366a646a73386170793479667634396e6c7375326c346473796d32737a633838376a673274726e6e6570376d63637868657773713532736830";
    let ascii = hex::decode(quote_token_addr)
        .expect("Failed to decode hex string");
    let bech = std::str::from_utf8(&ascii)
        .expect("Failed to convert bytes to string");

    let approve_contract: Bech32<FixedBytes<32>> =
        Bech32::from_str(bech)
            .unwrap();

    println!("Bech32 Address: {}", approve_contract);
    
    let cosmos_cfg = cosmos::Config {
        chain_id: ChainId::new("union-devnet-1"),
        ibc_host_contract_address: Bech32::from_str(
            "union1nk3nes4ef6vcjan5tz6stf9g8p08q2kgqysx6q5exxh89zakp0msq5z79t",
        )
        .unwrap(),
        keyring: KeyringConfig {
            name: "alice".into(),
            keys: vec![KeyringConfigEntry::Raw {
                name: "alice".into(),
                key: hex_literal::hex!(
                    "aa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
                )
                .to_vec(),
            }],
        },
        rpc_url: "http://localhost:26657".into(),
        gas_config: GasFillerConfig::Feemarket(FeemarketConfig {
            max_gas: 10000000,
            gas_multiplier: Some(1.4),
            denom: None,
        }),
        fee_recipient: None,
    };

    let evm_cfg = evm::Config {
        chain_id: ChainId::new("32382"),
        ibc_handler_address: hex!("ed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5").into(),
        multicall_address: hex!("84c4c2ee43ccfd523af9f78740256e0f60d38068").into(),
        rpc_url: "http://localhost:8545".into(),
        ws_url  : "ws://localhost:8546".into(),
        keyring: KeyringConfig {
            name: "alice".into(),
            keys: vec![KeyringConfigEntry::Raw {
                name: "alice".into(),
                key: hex!("4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77")
                    .to_vec(),
            }],
        },
        max_gas_price: None,
        fixed_gas_price: None,
        gas_multiplier: 2.0,
    };

    let src = cosmos::Module::new(cosmos_cfg.clone()).await?;
    let dst = evm::Module::new(evm_cfg.clone()).await?;


    // 3) now hand them to your library’s TestContext
    let ctx = TestContext::new(src, dst, 1).await?;

    // 4) invoke create_clients and inspect the two confirms
    // let (src_confirm, dst_confirm) = ctx
    //     .create_clients(
    //         Duration::from_secs(45),
    //         "ibc-cosmwasm",
    //         "trusted/evm/mpt",
    //         "ibc-solidity",
    //         "cometbls",
    //     )
    //     .await?;

    // println!("✅ src CreateClientConfirm = {:#?}", src_confirm);
    // println!("✅ dst CreateClientConfirm = {:#?}", dst_confirm);

    // let conn_confirm = ctx
    //     .open_connection::<cosmos::Module, evm::Module>(
    //         &ctx.src,
    //         src_confirm.client_id,
    //         &ctx.dst,
    //         dst_confirm.client_id,
    //         Duration::from_secs(180),
    //     )
    //     .await?;

    // println!(
    //     "✅ ConnectionOpenConfirm = src {} ↔ dst {}",
    //     conn_confirm.connection_id, conn_confirm.counterparty_connection_id,
    // );

    // let opened = ctx
    //     .open_channels(
    //         true,
    //         "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c"
    //             .as_bytes()
    //             .into(),
    //         hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5")
    //             .to_vec()
    //             .into(),
    //         conn_confirm.connection_id,
    //         "ucs03-zkgm-0".into(),
    //         1,
    //         Duration::from_secs(360),
    //     )
    //     .await?;

    // println!("Opened {} channels", opened);

    // let pair = ctx.get_channel().await.unwrap();


    let zkgm_evm_addr = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");
    let init_call = ZkgmERC20::initializeCall {
        _authority: hex!("6C1D11bE06908656D16EBFf5667F1C45372B7c89").into(),
        _minter: zkgm_evm_addr.into(),
        _name: "muno".into(),
        _symbol: "muno".into(),
        _decimals: 6u8.into(),
    };
    let img_metadata = FungibleAssetMetadata {
        implementation: hex!("999709eB04e8A30C7aceD9fd920f7e04EE6B97bA").to_vec().into(),
        initializer: init_call.abi_encode().into(),
    }.abi_encode_params();


    let img = keccak256(&img_metadata);

    // panic!("panicked");

    let pair = ChannelPair {
        src: 1.try_into().unwrap(),
        dest: 1.try_into().unwrap(),
    };
    // println!("Channel {} ↔ {}", pair.src, pair.dest);
    
    // let spender = hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD");

    // // let gov_token_addr = hex!("a513f3a432f575f1e8579cc456badac9c78d8b08");
    // let governance_token: GovernanceToken = ctx.dst
    //     .setup_governance_token(zkgm_evm_addr.into(),  pair.dest, img)
    //     .await?;

    // println!("✅ governance_token.unwrappedToken registered at: {:?}", governance_token.unwrappedToken);
    // println!("✅ governance_token.metadataImage registered at: {:?}", governance_token.metadataImage);

    // println!("✅ Governance token registered at: {:?}", deployed_erc20_addr);
    /*
✅ Governance token registered at: FixedBytes<20>(0x6c2bcc9c340595143c31b4e2b238ae8f0c04e572)
✅ Governance token registered at: FixedBytes<20>(0xa513f3a432f575f1e8579cc456badac9c78d8b08) */

    let snake_nft = ctx.dst.predict_stake_manager_address(zkgm_evm_addr.into()).await?;
    
    println!("✅ Stake manager address: {:?}", snake_nft);

    // // ctx.dst.basic_erc721_mint(snake_nft, U256::from(1u32), spender.into()).await?;

    let mut salt = [0u8; 32];
    rand::rng().fill_bytes(&mut salt);

    let quote_token_addr  = ctx.predict_wrapped_token_from_metadata_image_v2::<evm::Module>(
        &ctx.dst,
        zkgm_evm_addr.into(),
        ChannelId::new(NonZero::new(pair.dest).unwrap()),
        "muno".into(),
        img.into(),
    ).await.unwrap();

    println!("✅ Quote token address: {:?}", quote_token_addr);
    // panic!("panicked");

    // sending muno here
    let mut salt_bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut salt_bytes);

    let contract: Bech32<FixedBytes<32>> =
        Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
            .unwrap();

    let instruction_cosmos = Instruction {
        version: INSTR_VERSION_2,
        opcode: OP_FUNGIBLE_ASSET_ORDER,
        operand: FungibleAssetOrderV2 {
            sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
                .as_bytes()
                .into(),
            receiver: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                .to_vec()
                .into(),
            base_token: "muno".as_bytes().into(),
            base_amount: "10".parse().unwrap(),
            metadata_type: FUNGIBLE_ASSET_METADATA_TYPE_PREIMAGE,
            metadata: img_metadata.into(),
            quote_token: quote_token_addr.as_ref().to_vec().into(),
            quote_amount: "10".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let cw_msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: pair.src.try_into().unwrap(),
        timeout_height: 0u64.into(),
        timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
        salt: salt_bytes.into(),
        instruction: instruction_cosmos.abi_encode_params().into(),
    };
    let bin_msg: Vec<u8> = Encode::<Json>::encode(&cw_msg);

    let funds = vec![Coin {
        denom: "muno".into(),
        amount: "10".into(),
    }];

    // TODO: Here we should check the muno balance of sender account
    // Also token balanceOf the receiver account
    let recv_packet_data = ctx
        .send_and_recv_with_retry::<cosmos::Module, evm::Module>(
            &ctx.src,
            contract,
            (bin_msg, funds).into(),
            &ctx.dst,
            3,
            Duration::from_secs(20),
            Duration::from_secs(720),
        )
        .await;

    println!("Received packet data: {:?}", recv_packet_data);

    //appropve here
    println!("Calling approve on quote token: {:?}", quote_token_addr); // 0xc7484B8b13FdE71A7203876359f1484808DCCc4A

    let approve_tx_hash = ctx
        .dst
        .zkgmerc20_approve(
            quote_token_addr.into(),
            zkgm_evm_addr.into(),
            U256::from(100000000000u64),
        )
        .await?;

    println!("✅ Approve tx hash: {:?}", approve_tx_hash);
    println!("IMG: {:?}", img);
    
    let given_validator = "unionvaloper1qp4uzhet2sd9mrs46kemse5dt9ncz4k3xuz7ej";
    let instruction_from_evm_to_union = InstructionEvm {
        version: INSTR_VERSION_0,
        opcode:  OP_STAKE,
        operand: Stake {
            token_id:  "7".parse().unwrap(),
            // governance_token: governance_token.unwrappedToken,
            // governance_metadata_image: governance_token.metadataImage,
            governance_token: b"muno".into(),
            governance_metadata_image: img.into(),
            sender:   hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                    .to_vec()
                    .into(),
            beneficiary:  hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
                    .to_vec()
                    .into(),
            validator:    given_validator.as_bytes().into(),
            amount:   "1".parse().unwrap(),
        }
        .abi_encode_params()
        .into(),
    };

    let evm_provider = ctx.dst.get_provider().await;

    let ucs03_zkgm = UCS03Zkgm::new(zkgm_evm_addr.into(), evm_provider);

    rand::rng().fill_bytes(&mut salt);
    let call = ucs03_zkgm
        .send(
            pair.dest.try_into().unwrap(),
            0u64.into(),
            4294967295000000000u64.into(),
            salt.into(),
            instruction_from_evm_to_union.clone(),
        )
        .clear_decoder();
    // let call = call.with_cloned_provider();
    let recv_packet_data = ctx
        .send_and_recv_stake::<evm::Module, cosmos::Module>(
            &ctx.dst,
            zkgm_evm_addr.into(),
            call,
            &ctx.src,
            Duration::from_secs(360),
            given_validator.to_string()
        )
        .await;

    println!("Received packet data: {:?}", recv_packet_data);
    // union tarafinda delegate eventi aricaz
    // nft de verdigim tokenid benim mi diye bakcaz

    // panic!("panicked");
    
    
    // let mut salt_bytes: [u8; 32] = [0u8; 32];
    // rand::rng().fill_bytes(&mut salt_bytes);
    // let contract: Bech32<FixedBytes<32>> =
    //     Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
    //         .unwrap();
    // let funded_msg = (
    //     Encode::<Json>::encode(ucs03_zkgm::msg::ExecuteMsg::Send {
    //         channel_id: pair.dest.try_into().unwrap(),
    //         timeout_height: 0u64.into(),
    //         timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
    //         salt: salt_bytes.into(),
    //         instruction: Instruction {
    //             version: INSTR_VERSION_1,
    //             opcode: OP_FUNGIBLE_ASSET_ORDER,
    //             operand: FungibleAssetOrder {
    //                 sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
    //                     .as_bytes()
    //                     .into(),
    //                 receiver: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
    //                     .to_vec()
    //                     .into(),
    //                 base_token: "muno".as_bytes().into(),
    //                 base_amount: "10".parse().unwrap(),
    //                 base_token_symbol: "muno".into(),
    //                 base_token_name: "muno".into(),
    //                 base_token_decimals: 6,
    //                 base_token_path: "0".parse().unwrap(),
    //                 quote_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579")
    //                     .to_vec()
    //                     .into(),
    //                 quote_amount: "10".parse().unwrap(),
    //             }
    //             .abi_encode_params()
    //             .into(),
    //         }
    //         .abi_encode_params()
    //         .into(),
    //     }),
    //     vec![Coin {
    //         denom: "muno".into(),
    //         amount: "10".into(),
    //     }],
    // );

    // let packet_hash = ctx.src.send_ibc_transaction(contract, funded_msg).await?;
    // let recv_packet_data = ctx
    //     .dst
    //     .wait_for_packet_recv(packet_hash, Duration::from_secs(360))
    //     .await;

    // assert!(
    //     recv_packet_data.is_ok(),
    //     "Failed to send and receive packet: {:?}",
    //     recv_packet_data.err()
    // );

    // let mut salt_bytes = [0u8; 32];
    // rand::rng().fill_bytes(&mut salt_bytes);

    // let instruction_from_evm_to_union = InstructionEvm {
    //     version: INSTR_VERSION_1,
    //     opcode: OP_FUNGIBLE_ASSET_ORDER,
    //     operand: FungibleAssetOrder {
    //         sender: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD").to_vec().into(),
    //         receiver: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2".as_bytes().into(),
    //         base_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579").to_vec().into(),
    //         base_amount: "1".parse().unwrap(),
    //         base_token_symbol: "muno".into(),
    //         base_token_name: "muno".into(),
    //         base_token_decimals: 6,
    //         base_token_path: "1".parse().unwrap(),
    //         quote_token:  "muno".into(),
    //         quote_amount: "1".parse().unwrap(),
    //     }
    //     .abi_encode_params()
    //     .into(),
    // };

    // let ucs03_addr_on_evm = hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5");
    // let eth = evm::Module::new(evm_cfg.clone()).await?;
    // let union = cosmos::Module::new(cosmos_cfg.clone()).await?;

    // let send_call_struct = UCS03Zkgm::sendCall {
    //     channelId: 1.try_into().unwrap(),
    //     timeoutTimestamp: 4294967295000000000u64.into(),
    //     timeoutHeight: 0u64.into(),
    //     salt: salt_bytes.into(),
    //     instruction: instruction_from_evm_to_union.clone(),
    // };

    // let hash_val = eth.send_zkgm_transaction(
    //     ucs03_addr_on_evm.into(),
    //     send_call_struct
    // ).await;

    // println!("Hash Value: {:?}", hash_val);

    // let recv_packet = union.wait_for_packet_recv(
    //     hash_val.unwrap(),
    //     Duration::from_secs(280),
    // ).await;

    // println!("Received Packet: {:?}", recv_packet);

    // let contract: Bech32<FixedBytes<32>> = Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
    //     .unwrap();
    // let funded_msgs = vec![(
    //     Box::new(ucs03_zkgm::msg::ExecuteMsg::Send {
    //         channel_id: 1.try_into().unwrap(),
    //         timeout_height: 0u64.into(),
    //         timeout_timestamp: voyager_sdk::primitives::Timestamp::from_secs(u32::MAX.into()),
    //         salt: salt_bytes.into(),
    //         instruction: Instruction {
    //             version: INSTR_VERSION_1,
    //             opcode: OP_FUNGIBLE_ASSET_ORDER,
    //             operand: FungibleAssetOrder {
    //                 sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2".as_bytes().into(),
    //                 receiver: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD").to_vec().into(),
    //                 base_token: "muno".as_bytes().into(),
    //                 base_amount: "10".parse().unwrap(),
    //                 base_token_symbol: "muno".into(),
    //                 base_token_name: "muno".into(),
    //                 base_token_decimals: 6,
    //                 base_token_path: "0".parse().unwrap(),
    //                 quote_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579").to_vec().into(),
    //                 quote_amount: "10".parse().unwrap(),
    //             }
    //             .abi_encode_params()
    //             .into(),
    //         }
    //         .abi_encode_params()
    //         .into(),
    //     }),
    //     vec![Coin {
    //         denom: "muno".into(),
    //         amount: "10".into(),
    //     }],
    // )];

    // let recv_packet_data = ctx.send_and_recv(
    //     true, // send from source
    //     contract,
    //     funded_msgs,
    //     Duration::from_secs(360),
    // ).await;
    // assert!(recv_packet_data.is_ok(), "Failed to send and receive packet: {:?}", recv_packet_data.err());

    // // 4) invoke create_clients and inspect the two confirms
    // let (src_confirm, dst_confirm) = ctx
    //     .create_clients(
    //         Duration::from_secs(45),
    //         "ibc-cosmwasm",
    //         "trusted/evm/mpt",
    //         "ibc-solidity",
    //         "cometbls",
    //     )
    //     .await?;

    // println!("✅ src CreateClientConfirm = {:#?}", src_confirm);
    // println!("✅ dst CreateClientConfirm = {:#?}", dst_confirm);

    // let conn_confirm = ctx
    //     .open_connection(
    //         true,
    //         src_confirm.client_id,
    //         dst_confirm.client_id,
    //         Duration::from_secs(180),
    //     )
    //     .await?;

    // println!(
    //     "✅ ConnectionOpenConfirm = src {} ↔ dst {}",
    //     conn_confirm.connection_id,
    //     conn_confirm.counterparty_connection_id,
    // );

    // let opened = ctx
    //     .open_channels(
    //         true,
    //         "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c".as_bytes().into(),
    //         hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").to_vec().into(),
    //         conn_confirm.connection_id,
    //         "ucs03-zkgm-0".into(),
    //         1,
    //         Duration::from_secs(240),
    //     )
    //     .await?;

    // println!("Opened {} channels", opened);

    // let conn_confirm_pt2 = ctx
    //     .open_connection(
    //         false,
    //         src_confirm.client_id,
    //         dst_confirm.client_id,
    //         Duration::from_secs(180),
    //     )
    //     .await?;

    // println!(
    //     "✅ ConnectionOpenConfirm = src {} ↔ dst {}",
    //     conn_confirm_pt2.connection_id,
    //     conn_confirm_pt2.counterparty_connection_id,
    // );

    Ok(())
}

// #[tokio::main]
// async fn main() -> anyhow::Result<()> {
//     let union_config = cosmos::Config {
//         chain_id: ChainId::new("union-devnet-1"),
//         ibc_host_contract_address: Bech32::from_str(
//             "union1nk3nes4ef6vcjan5tz6stf9g8p08q2kgqysx6q5exxh89zakp0msq5z79t",
//         )
//         .unwrap(),
//         keyring: KeyringConfig {
//             name: "alice".into(),
//             keys: vec![KeyringConfigEntry::Raw {
//                 name: "alice".into(),
//                 key: hex_literal::hex!(
//                     "aa820fa947beb242032a41b6dc9a8b9c37d8f5fbcda0966b1ec80335b10a7d6f"
//                 )
//                 .to_vec(),
//             }],
//         },
//         rpc_url: "http://localhost:26657".into(),
//         gas_config: GasFillerConfig::Feemarket(FeemarketConfig {
//             max_gas: 10000000,
//             gas_multiplier: Some(1.4),
//             denom: None,
//         }),
//         fee_recipient: None,
//     };

// //     let union = cosmos::Module::new(union_config).await?;

//     let evm_config = evm::Config {
//         chain_id: ChainId::new("32382"),
//         ibc_handler_address: hex!("ed2af2aD7FE0D92011b26A2e5D1B4dC7D12A47C5").into(),
//         multicall_address: hex!("84c4c2ee43ccfd523af9f78740256e0f60d38068").into(),
//         rpc_url: "http://localhost:8545".into(),
//         keyring: KeyringConfig {
//             name: "alice".into(),
//             keys: vec![KeyringConfigEntry::Raw {
//                 name: "alice".into(),
//                 key: hex!("4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77")
//                     .to_vec(),
//             }],
//         },
//         max_gas_price: None,
//         fixed_gas_price: None,
//         gas_multiplier: 2.0,
//     };

//     let eth = evm::Module::new(evm_config).await?;

//     voyager::init_fetch(union.chain_id.clone())?;
//     voyager::init_fetch(eth.chain_id.clone())?;

//     voyager::create_client(
//         eth.chain_id.clone(),
//         union.chain_id.clone(),
//         "ibc-solidity".into(),
//         "cometbls".into(),
//     )?;

//     let res = eth.wait_for_create_client(Duration::from_secs(45)).await;

//     let counterparty_client_id = match res {
//         Ok(confirm) => {
//             println!(
//                 "✅ got create client result on EVM. client_id: {}",
//                 confirm.client_id,
//             );
//             confirm.client_id
//         }
//         Err(err) => {
//             eprintln!("⚠️  error waiting for create-client-confirm: {}", err);
//             return Ok(());
//         }
//     };

//     voyager::create_client(
//         union.chain_id.clone(),
//         eth.chain_id.clone(),
//         "ibc-cosmwasm".into(),
//         "trusted/evm/mpt".into(),
//     )?;

//     let res = union.wait_for_create_client_id(Duration::from_secs(45)).await;
//     let client_id: u32 = match res {
//         Ok(confirm) => {
//             println!(
//                 "✅ got create client result on Cosmos. client_id: {}",
//                 confirm.client_id,
//             );
//             confirm.client_id
//         }
//         Err(err) => {
//             eprintln!("⚠️  error waiting for create-client-confirm: {}", err);
//             return Ok(());
//         }
//     };

//     std::thread::sleep(Duration::from_secs(5));

//     voyager::connection_open(union.chain_id.clone(), client_id, counterparty_client_id)?;

//     let res = eth.wait_for_connection_open_confirm(Duration::from_secs(180)).await;

//     let connection_id = match res {
//         Ok(confirm) => {
//             println!(
//                 "✅ got connection confirm: {} ↔ {}",
//                 confirm.connection_id,
//                 confirm.counterparty_connection_id,
//             );
//             confirm.counterparty_connection_id
//         }
//         Err (err) => {
//             println!("Error occured when waiting for connection open confirm. Err: {}", err);
//             return Ok(());
//         }
//     };

//     let channel_pool = channel_provider::ChannelPool::new();

//     let opened = channel_pool
//         .open_channels(
//             voyager::channel_open,                                       // fn pointer
//             |timeout: Duration| {                                        // map to ChannelConfirm
//                 let eth = &eth;                                         // capture `eth`
//                 async move {
//                     let ev = eth.wait_for_channel_open_confirm(timeout).await?;
//                     Ok(ChannelConfirm {
//                         channel_id: ev.channel_id,
//                         counterparty_channel_id: ev.counterparty_channel_id,
//                     })
//                 }
//             },
//             union.chain_id.clone(),
//             "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c".as_bytes().into(),
//             eth.chain_id.clone(),
//             hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").to_vec().into(),
//             connection_id,
//             "ucs03-zkgm-0".into(),
//             1,
//             Duration::from_secs(240) // 1 attempt, 240 seconds timeout
//         )
//         .await?;

//     println!("Opened {} channels", opened);

//     let channel_id: ChannelId = match channel_pool
//         .get_channel(&union.chain_id, &eth.chain_id)
//         .await
//     {
//         Some(channel) => {
//             println!(
//                 "Channel {} ↔ {}",
//                 channel.src,
//                 channel.dest,
//             );
//             channel.src.try_into().unwrap()
//         }
//         None => {
//             eprintln!("⚠️  No more channels available");
//             panic!("no channel to send on");
//         }
//     };

//     let mut salt_bytes = [0u8; 32];
//     rand::rng().fill_bytes(&mut salt_bytes);

//     let cosmos::IbcEvent::WasmPacketSend { packet_hash, .. } = union
//         .send_ibc_transaction(
//             Bech32::from_str("union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c")
//                 .unwrap(),
//             vec![(
//                 Box::new(ucs03_zkgm::msg::ExecuteMsg::Send {
//                     channel_id: channel_id,
//                     timeout_height: 0u64.into(),
//                     timeout_timestamp: Timestamp::from_secs(u32::MAX.into()),
//                     salt: salt_bytes.into(),
//                     instruction: Instruction {
//                         version: INSTR_VERSION_1,
//                         opcode: OP_FUNGIBLE_ASSET_ORDER,
//                         operand: FungibleAssetOrder {
//                             sender: "union1jk9psyhvgkrt2cumz8eytll2244m2nnz4yt2g2"
//                                 .as_bytes()
//                                 .into(),
//                             receiver: hex!("Be68fC2d8249eb60bfCf0e71D5A0d2F2e292c4eD")
//                                 .to_vec()
//                                 .into(),
//                             base_token: "muno".as_bytes().into(),
//                             base_amount: "10".parse().unwrap(),
//                             base_token_symbol: "muno".into(),
//                             base_token_name: "muno".into(),
//                             base_token_decimals: 6,
//                             base_token_path: "0".parse().unwrap(),
//                             quote_token: hex!("16628cB81ffDA9B8470e16299eFa5F76bF45A579")
//                                 .to_vec()
//                                 .into(),
//                             quote_amount: "10".parse().unwrap(),
//                         }
//                         .abi_encode_params()
//                         .into(),
//                     }
//                     .abi_encode_params()
//                     .into(),
//                 }),
//                 vec![Coin {
//                     denom: "muno".into(),
//                     amount: "10".into(),
//                 }],
//             )],
//         )
//         .await
//         .unwrap()
//         .unwrap() else { todo!("unexpected event type") };

//     let recv = match eth
//         .wait_for_packet_recv(packet_hash, Duration::from_secs(280))
//         .await
//     {
//         Ok(ev) => {
//             println!("✅ packet received: {:?}", ev);
//             ev
//         }
//         Err(err) => {
//             eprintln!("⚠️  error waiting for PacketRecv: {}", err);
//             return Ok(());
//         }
//     };

//     Ok(())
// }

/*
pub async fn open_channels(
    &self,
    src_port: Bytes,
    dst_port: Bytes,
    connection_id: u32,
    count: usize,
    duration: Duration,
) -> anyhow::Result<usize> {
  */

// async fn test_channels(
//     pool: &channel_provider::ChannelPool,
//     src: &ChainId,
//     dst: &ChainId,
//     max: usize,
//     release_at: Option<usize>,
//     eth: &evm::Module,
//     union: &cosmos::Module,
//     connection_id: u32,
// ) {

//     let opened = pool
//         .open_channels(
//             voyager::channel_open,
//             |timeout: Duration| {
//                 let eth = &eth;
//                 async move {
//                     let ev = eth.wait_for_channel_open_confirm(timeout).await?;
//                     Ok(ChannelConfirm {
//                         channel_id: ev.channel_id,
//                         counterparty_channel_id: ev.counterparty_channel_id,
//                     })
//                 }
//             },
//             union.chain_id.clone(),
//             "union1rfz3ytg6l60wxk5rxsk27jvn2907cyav04sz8kde3xhmmf9nplxqr8y05c".as_bytes().into(),
//             eth.chain_id.clone(),
//             hex!("05fd55c1abe31d3ed09a76216ca8f0372f4b2ec5").to_vec().into(),
//             connection_id,
//             "ucs03-zkgm-0".into(),
//             8,
//             Duration::from_secs(240*8) // 8 attempts, 240 seconds each
//         )
//         .await;

//     for idx in 1..=max {
//         println!("Attempting to get channel #{}", idx);
//         match pool.get_channel(src, dst).await {
//             Some(channel) => {
//                 println!("Channel {}: {} ↔ {}", idx, channel.src, channel.dest);
//                 if release_at.map_or(false, |n| n == idx) {
//                     println!("Releasing channel #{}", idx);
//                     pool.release_channel(src, dst, channel).await;
//                 }
//             }
//             None => {
//                 eprintln!("⚠️  No more channels available at iteration {}", idx);
//                 break;
//             }
//         }
//     }
// }
