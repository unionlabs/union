use crate::errors::StdError;

/// A record of a key-value storage that is created through an iterator API.
/// The first element (key) is always raw binary data. The second element
/// (value) is binary by default but can be changed to a custom type. This
/// allows contracts to reuse the type when deserializing database records.
pub type Record<V = Vec<u8>> = (Vec<u8>, V);

#[derive(Copy, Clone, PartialEq, Eq)]
// We assign these to integers to provide a stable API for passing over FFI (to wasm and Go)
pub enum Order {
    Ascending = 1,
    Descending = 2,
}

impl TryFrom<i32> for Order {
    type Error = StdError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Order::Ascending),
            2 => Ok(Order::Descending),
            _ => Err(StdError::generic_err("Order must be 1 or 2")),
        }
    }
}

impl From<Order> for i32 {
    fn from(original: Order) -> i32 {
        original as _
    }
}
