#![cfg(feature = "sync")]

use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::StreamExt;

#[tokio::test]
async fn message_not_twice() {
    let (tx, rx) = watch::channel("hello");

    let mut counter = 0;
    let mut stream = WatchStream::new(rx).map(move |payload| {
        println!("{}", payload);
        if payload == "goodbye" {
            counter += 1;
        }
        if counter >= 2 {
            panic!("too many goodbyes");
        }
    });

    let task = tokio::spawn(async move { while stream.next().await.is_some() {} });

    // Send goodbye just once
    tx.send("goodbye").unwrap();

    drop(tx);
    task.await.unwrap();
}
