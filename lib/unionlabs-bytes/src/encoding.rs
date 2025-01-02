use core::fmt;

trait Sealed {}

#[expect(private_bounds)]
pub trait Encoding: Sealed {
    type Error: core::error::Error + 'static;

    /// Encode `bytes` and write the value to `f`.
    ///
    /// # Errors
    ///
    /// This method should only fail if writing to `f` fails. See the documentation on [`std::fmt::Display::fmt`] for more information.
    fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result;

    /// Decode the given data slice.
    ///
    /// # Errors
    ///
    /// This method is expected to error on any decoding error encountered.
    fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error>;

    /// Decode the given data slice into the given buffer.
    ///
    /// # Errors
    ///
    /// This method is expected to error if `out` is not the exact length required for the decoded data, or on any decoding error encountered.
    fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Self::Error>;
}

pub struct HexPrefixed;
impl Sealed for HexPrefixed {}
impl Encoding for HexPrefixed {
    type Error = HexPrefixedFromStrError;

    fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("0x{}", hex::encode(bytes)))
    }

    fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error> {
        let data = data
            .as_ref()
            .strip_prefix(b"0x")
            .ok_or(HexPrefixedFromStrError::MissingPrefix)?;

        hex::decode(data).map_err(HexPrefixedFromStrError::InvalidHex)
    }

    fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Self::Error> {
        let data = data
            .as_ref()
            .strip_prefix(b"0x")
            .ok_or(HexPrefixedFromStrError::MissingPrefix)?;

        hex::decode_to_slice(data, out).map_err(HexPrefixedFromStrError::InvalidHex)
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum HexPrefixedFromStrError {
    #[error("invalid hex")]
    InvalidHex(#[from] hex::FromHexError),
    #[error("missing 0x prefix")]
    MissingPrefix,
}

pub struct HexUnprefixed;
impl Sealed for HexUnprefixed {}
impl Encoding for HexUnprefixed {
    type Error = hex::FromHexError;

    fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(bytes))
    }

    fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error> {
        hex::decode(data)
    }

    fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Self::Error> {
        hex::decode_to_slice(data, out)
    }
}

pub struct Base64;
impl Sealed for Base64 {}
impl Encoding for Base64 {
    type Error = Base64Error;

    fn fmt(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use base64::prelude::*;

        f.write_str(&BASE64_STANDARD.encode(bytes))
    }

    fn decode<T: AsRef<[u8]>>(data: T) -> Result<Vec<u8>, Self::Error> {
        use base64::prelude::*;

        BASE64_STANDARD
            .decode(data)
            .map_err(Base64Error::InvalidEncoding)
    }

    fn decode_to_slice<T: AsRef<[u8]>>(data: T, out: &mut [u8]) -> Result<(), Base64Error> {
        use base64::prelude::*;

        let vec = BASE64_STANDARD.decode(data)?;

        if vec.len() == out.len() {
            out.copy_from_slice(&vec);

            Ok(())
        } else {
            Err(Base64Error::InvalidLength {
                expected_len: out.len(),
                found_len: vec.len(),
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Base64Error {
    #[error("invalid encoding")]
    InvalidEncoding(#[from] base64::DecodeError),
    #[error("invalid length (expected {expected_len}, found {found_len})")]
    InvalidLength {
        expected_len: usize,
        found_len: usize,
    },
}
