use crate::maybestd::{
    io::{Result, Write},
    vec::Vec,
};
use crate::BorshSerialize;

/// Serialize an object into a vector of bytes.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: BorshSerialize + ?Sized,
{
    value.try_to_vec()
}

/// Serializes an object directly into a `Writer`.
pub fn to_writer<T, W: Write>(mut writer: W, value: &T) -> Result<()>
where
    T: BorshSerialize + ?Sized,
{
    value.serialize(&mut writer)
}
