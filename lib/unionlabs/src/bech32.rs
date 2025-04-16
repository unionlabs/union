use core::{fmt, marker::PhantomData, str::FromStr};

use serde::{de, Deserialize, Serialize};

use crate::{primitives::Bytes, ErrorReporter};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bech32<Data = Bytes, Hrp = String> {
    // TODO: Validate the HRP
    hrp: Hrp,
    data: Data,
}

impl<Data, Hrp> Bech32<Data, Hrp> {
    pub const fn new(hrp: Hrp, data: Data) -> Self {
        Self { hrp, data }
    }

    pub const fn hrp(&self) -> &Hrp {
        &self.hrp
    }

    pub const fn data(&self) -> &Data {
        &self.data
    }

    pub fn map_data<NewData>(self, f: impl FnOnce(Data) -> NewData) -> Bech32<NewData, Hrp> {
        Bech32::<NewData, Hrp>::new(self.hrp, f(self.data))
    }

    pub fn map_hrp<NewHrp>(self, f: impl FnOnce(Hrp) -> NewHrp) -> Bech32<Data, NewHrp> {
        Bech32::<Data, NewHrp>::new(f(self.hrp), self.data)
    }
}

impl<Data, Hrp> Bech32<Data, Hrp>
where
    Data: TryFrom<Vec<u8>>,
    Hrp: From<String>,
{
    pub fn decode(encoded: impl AsRef<str>) -> Result<Self, Bech32DecodeError<Data::Error>> {
        let (hrp, bytes) =
            subtle_encoding::bech32::decode(encoded).map_err(Bech32DecodeError::Decode)?;

        Ok(Self {
            hrp: Hrp::from(hrp),
            data: bytes.try_into().map_err(Bech32DecodeError::Data)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Bech32DecodeError<E> {
    #[error("error decoding bech32 string")]
    Decode(#[cfg_attr(feature = "std", source)] subtle_encoding::Error),
    #[error("unable to convert decoded data")]
    Data(#[source] E),
}

impl<Data, Hrp> FromStr for Bech32<Data, Hrp>
where
    Data: TryFrom<Vec<u8>>,
    Hrp: From<String>,
{
    type Err = Bech32DecodeError<Data::Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::decode(s)
    }
}

impl<Hrp, Data> fmt::Display for Bech32<Data, Hrp>
where
    Hrp: AsRef<str>,
    Data: AsRef<[u8]>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&subtle_encoding::bech32::encode(
            self.hrp.as_ref(),
            self.data.as_ref(),
        ))
    }
}

impl<Data, Hrp> Serialize for Bech32<Data, Hrp>
where
    Data: AsRef<[u8]>,
    Hrp: AsRef<str>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de, Data, Hrp> Deserialize<'de> for Bech32<Data, Hrp>
where
    Data: TryFrom<Vec<u8>, Error: core::error::Error + 'static>,
    Hrp: From<String>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Bech32Visitor<Data, Hrp>(PhantomData<fn() -> (Data, Hrp)>);

        impl<'de, Data, Hrp> de::Visitor<'de> for Bech32Visitor<Data, Hrp>
        where
            Data: TryFrom<Vec<u8>, Error: core::error::Error + 'static>,
            Hrp: From<String>,
        {
            type Value = Bech32<Data, Hrp>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a bech32 encoded string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Bech32::decode(v).map_err(|e| de::Error::custom(ErrorReporter(e)))
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&v)
            }
        }

        deserializer.deserialize_str(Bech32Visitor(PhantomData))
    }
}

#[cfg(feature = "bincode")]
impl<Hrp, Data> bincode::Encode for Bech32<Data, Hrp>
where
    Hrp: AsRef<str>,
    Data: AsRef<[u8]>,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.to_string().encode(encoder)
    }
}

#[cfg(feature = "bincode")]
impl<Context, Data, Hrp> bincode::Decode<Context> for Bech32<Data, Hrp>
where
    Data: TryFrom<Vec<u8>>,
    Hrp: From<String>,
{
    fn decode<D: bincode::de::Decoder<Context = Context>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        <String as bincode::Decode<Context>>::decode(decoder).and_then(|s| {
            s.parse::<Self>()
                .map_err(|e| bincode::error::DecodeError::OtherString(e.to_string()))
        })
    }
}

#[cfg(feature = "bincode")]
impl<'de, Context, Data, Hrp> bincode::BorrowDecode<'de, Context> for Bech32<Data, Hrp>
where
    Data: TryFrom<Vec<u8>>,
    Hrp: From<String>,
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        bincode::Decode::decode(decoder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONST_BECH32: Bech32<[u8; 4], &'static str> = Bech32::new("hrp", *b"data");

    #[test]
    fn const_to_string() {
        assert_eq!(CONST_BECH32.to_string(), "hrp1v3shgcgtzrn02");
    }

    #[test]
    fn string_roundtrip() {
        let s = "union1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqavqtmv";

        let decoded = s.parse::<Bech32>().unwrap();

        assert_eq!(s, decoded.to_string());
    }

    #[test]
    fn json_roundtrip() {
        let s = r#""union1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqavqtmv""#;

        let decoded = serde_json::from_str::<Bech32>(s).unwrap();

        assert_eq!(s, serde_json::to_string(&decoded).unwrap());
    }
}
