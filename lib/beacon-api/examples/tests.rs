use beacon_api::client::{BeaconApiClient, BlockId};
use unionlabs::ethereum::config::Mainnet;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    do_main().await
}

async fn do_main() {
    let client = BeaconApiClient::new("https://lodestar-sepolia.chainsafe.io".to_string())
        .await
        .unwrap();

    // genesis
    let block = client.block(BlockId::Slot(5772606)).await.unwrap();

    dbg!(block);

    // client.header(BlockId::Genesis).await.unwrap();

    // // head
    // client.block(BlockId::Head).await.unwrap();
    // client.header(BlockId::Head).await.unwrap();

    // // finalized
    // let finalized_block = client.block(BlockId::Finalized).await.unwrap();
    // let finalized_header = client.header(BlockId::Finalized).await.unwrap();

    // dbg!(&finalized_block);
    // dbg!(&finalized_header);

    // // slot
    // client
    //     .block(BlockId::Slot(finalized_block.data.message.slot - 1))
    //     .await
    //     .unwrap();
    // let slot_header = client.header(BlockId::Slot(32)).await.unwrap();

    // // hash
    // client
    //     .block(BlockId::Hash(slot_header.data.root))
    //     .await
    //     .unwrap();
    // client
    //     .header(BlockId::Hash(slot_header.data.root))
    //     .await
    //     .unwrap();

    // // bootstrap
    // client.bootstrap(finalized_header.data.root).await.unwrap();

    // // finality update
    // // client.finality_update().await.unwrap();
}
