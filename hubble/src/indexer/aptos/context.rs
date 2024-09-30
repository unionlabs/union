use std::fmt::Display;

use url::Url;

#[derive(Clone)]
pub struct AptosContext {
    pub rpc_urls: Vec<Url>,
    pub tx_search_max_page_size: u16,
}

impl Display for AptosContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rpcs: {}, tx_search_max_page_size: {}",
            to_indexed_url_string(&self.rpc_urls),
            self.tx_search_max_page_size,
        )
    }
}

fn to_indexed_url_string(urls: &[Url]) -> String {
    urls.iter()
        .enumerate()
        .map(|(index, url)| format!("{}: {}", index, url.as_str()))
        .collect::<Vec<_>>()
        .join(", ")
}
