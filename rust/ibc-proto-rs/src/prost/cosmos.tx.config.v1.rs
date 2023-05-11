/// Config is the config object of the x/auth/tx package.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Config {
    /// skip_ante_handler defines whether the ante handler registration should be skipped in case an app wants to override
    /// this functionality.
    #[prost(bool, tag = "1")]
    pub skip_ante_handler: bool,
    /// skip_post_handler defines whether the post handler registration should be skipped in case an app wants to override
    /// this functionality.
    #[prost(bool, tag = "2")]
    pub skip_post_handler: bool,
}
