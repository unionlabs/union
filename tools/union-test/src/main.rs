use std::{str::FromStr, time::Duration};

use alloy::sol_types::SolValue;
use concurrent_keyring::{KeyringConfig, KeyringConfigEntry};
use cosmos::{FeemarketConfig, GasFillerConfig};
use hex_literal::hex;

use unionlabs::{
    bech32::Bech32
};
use voyager_sdk::{anyhow, primitives::ChainId};
use union_test::{TestContext, cosmos, evm};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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


    let src = cosmos::Module::new(cosmos_cfg).await?;
    let dst = evm::Module::new(evm_cfg).await?;

    // 3) now hand them to your library’s TestContext
    let ctx = TestContext::new(src, dst).await?;

    // 4) invoke create_clients and inspect the two confirms
    let (src_confirm, dst_confirm) = ctx
        .create_clients(
            Duration::from_secs(45),
            "ibc-cosmwasm",  
            "trusted/evm/mpt",
            "ibc-solidity",
            "cometbls",
        )
        .await?;

    println!("✅ src CreateClientConfirm = {:#?}", src_confirm);
    println!("✅ dst CreateClientConfirm = {:#?}", dst_confirm);

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
//     rand::thread_rng().fill_bytes(&mut salt_bytes);


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
