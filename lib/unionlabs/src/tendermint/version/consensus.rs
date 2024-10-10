use macros::model;

#[derive(Default)]
#[model(proto(raw(protos::tendermint::version::Consensus), into, from))]
pub struct Consensus {
    #[serde(with = "::serde_utils::string")]
    pub block: u64,
    // REVIEW: Why default?
    #[serde(with = "::serde_utils::string", default)]
    pub app: u64,
}

impl From<protos::tendermint::version::Consensus> for Consensus {
    fn from(value: protos::tendermint::version::Consensus) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
}

impl From<Consensus> for protos::tendermint::version::Consensus {
    fn from(value: Consensus) -> Self {
        Self {
            block: value.block,
            app: value.app,
        }
    }
}
