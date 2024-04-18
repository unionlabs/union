use scroll_rpc::{BlockId, JsonRpcClient};
use unionlabs::uint::U256;

#[tokio::test]
#[ignore = "needs network"]
async fn get_proof() {
    let _ = tracing_subscriber::fmt::try_init();

    dbg!(JsonRpcClient::new("wss://sepolia-rpc.scroll.io")
        .await
        .unwrap()
        .get_proof(
            "0x58865036D143605698884D7dB32C808B4C7AFBe7"
                .parse()
                .unwrap(),
            [U256::from_be_hex("0x1").unwrap()],
            BlockId::Number(2994185)
        )
        .await
        .unwrap());
}
