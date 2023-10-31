use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::StrBuf;

impl<S: Sized> Serialize for StrBuf<S> {
    #[inline]
    fn serialize<SER: Serializer>(&self, ser: SER) -> Result<SER::Ok, SER::Error> {
        ser.serialize_str(self.as_str())
    }
}

struct StrBufVisitor<S>(core::marker::PhantomData<S>);

impl<'de, S: Sized> serde::de::Visitor<'de> for StrBufVisitor<S> {
    type Value = StrBuf<S>;

    #[inline(always)]
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("a string buffer")
    }

    #[inline]
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        if v.len() <= Self::Value::capacity() {
            let mut result = Self::Value::new();
            unsafe {
                result.push_str_unchecked(v);
            }
            Ok(result)
        } else {
            Err(serde::de::Error::custom(format_args!("Exceeds buffer capacity({} bytes)", Self::Value::capacity())))
        }
    }
}

impl<'a, S: Sized> Deserialize<'a> for StrBuf<S> {
    #[inline]
    fn deserialize<D: Deserializer<'a>>(des: D) -> Result<Self, D::Error> {
        des.deserialize_str(StrBufVisitor(core::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use crate::StrBuf;

    use serde::de::Deserialize;
    use serde::de::value::{BorrowedStrDeserializer, Error as ValueError};

    #[test]
    fn should_error_one_exceeding_capacity() {
        let des = BorrowedStrDeserializer::<ValueError>::new("lolka");
        let res = StrBuf::<[u8;4]>::deserialize(des);
        assert!(res.is_err());
    }

    #[test]
    fn should_ok_within_capacity() {
        let des = BorrowedStrDeserializer::<ValueError>::new("lolka");
        let res = StrBuf::<[u8;6]>::deserialize(des).expect("Unexpected fail");
        assert_eq!(res.as_str(), "lolka");
    }
}
