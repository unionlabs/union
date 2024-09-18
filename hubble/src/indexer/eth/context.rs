use std::fmt::Display;

use url::Url;

#[derive(Clone)]
pub struct EthContext {
    pub urls: Vec<Url>,
}

impl Display for EthContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "urls: {}",
            self.urls
                .iter()
                .enumerate()
                .map(|(index, url)| format!("{}: {}", index, url.as_str().to_string()))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
