use std::fmt::Display;

use regex::Regex;
use url::Url;

#[derive(Clone)]
pub struct TmContext {
    pub rpc_urls: Vec<Url>,
    pub grpc_urls: Vec<Url>,
    pub filter: Option<Regex>,
    pub tx_search_max_page_size: u8,
    pub client_tracking: bool,
}

impl Display for TmContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "rpcs: {}, grpcs: {}, filter: {}, tx_search_max_page_size: {}",
            to_indexed_url_string(&self.rpc_urls),
            to_indexed_url_string(&self.grpc_urls),
            match &self.filter {
                Some(regex) => regex.as_str(),
                None => "-",
            },
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
