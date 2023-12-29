#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

pub mod metering;

// The most commonly used symbol are exported at top level of the
// module. Others are available via modules,
// e.g. `wasmer_middlewares::metering::get_remaining_points`
pub use metering::Metering;
