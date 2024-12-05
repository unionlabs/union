use std::fmt::Display;

use url::Url;

#[derive(Clone)]
pub struct EthContext {
    pub rpc_urls: Vec<Url>,
    pub client_tracking: bool,
}

impl Display for EthContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rpc_urls: {}",
            self.rpc_urls
                .iter()
                .enumerate()
                .map(|(index, url)| format!("{}: {}", index, url.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
