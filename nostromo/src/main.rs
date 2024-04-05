use axum::{routing::get, Router};

enum NftMeta {
    ERC721(ERC721Meta),
    SG721(SG721Meta),
}

struct ERC721Meta {}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
