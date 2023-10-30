use std::fmt::Debug;

use serde::de::DeserializeOwned;

/// A marker trait for query response types.
///
/// Those types have in common that they should be `#[non_exhaustive]` in order
/// to allow adding fields in a backwards compatible way. In contracts they are
/// only constructed through deserialization. We want to make it hard for
/// contract developers to construct those types themselves as this is most likely
/// not what they should do.
///
/// In hosts they are constructed as follows:
/// - wasmvm: Go types with the same JSON layout
/// - multi-test/cw-sdk: create a default instance and mutate the fields
///
/// This trait is crate-internal and can change any time.
pub(crate) trait QueryResponseType: DeserializeOwned + Debug + PartialEq + Clone {}
