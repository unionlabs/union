use near_jsonrpc_client::methods;

#[tokio::main]
async fn main() {
    let signer = near_crypto::InMemorySigner::from_secret_key(signer_account_id, signer_secret_key);

    let access_key_query_response = client
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await?;
    // let rpc = near_jsonrpc_client::JsonRpcClient::connect("localhost:3030");

    // rpc.call(methods::send_tx::RpcSendTransactionRequest { signed_transaction: todo!(), wait_until:  })

    println!("Hello, world!");
}
