use std::time::Duration;

use hubble::hasura::{Datastore, GetLatestQueue, HasuraDataStore};
use reqwest::StatusCode;

#[tokio::main]
async fn main() {
    let [url, secret, voyager_url] = std::env::args()
        .skip(1)
        .collect::<Vec<_>>()
        .try_into()
        .expect("usage: voyager-sidecar <hasura-url> <hasura-secret> <voyager-url>");

    let client = reqwest::Client::new();

    let hasura_client = HasuraDataStore::new(
        client.clone(),
        url.parse().expect("first arg must be a url"),
        secret,
    );

    loop {
        if client
            .get(format!("{voyager_url}/health"))
            .send()
            .await
            .unwrap()
            .status()
            == StatusCode::OK
        {
            break;
        }

        println!("unable to reach voyager, trying again in 3 seconds");

        tokio::time::sleep(Duration::from_secs(3)).await
    }

    let [msg] = hasura_client
        .do_post::<GetLatestQueue>(hubble::hasura::get_latest_queue::Variables {})
        .await
        .expect("unable to fetch latest message from hasura")
        .data
        .expect("unable to read hasura response")
        .demo_queue
        .try_into()
        .expect("recieved more than one message from demo queue");

    println!("sending message to voyager: {}", msg.item.to_string());

    client
        .post(format!("{voyager_url}/msg"))
        .body(msg.item.to_string())
        .send()
        .await
        .expect("unable to send message to voyager");
}
