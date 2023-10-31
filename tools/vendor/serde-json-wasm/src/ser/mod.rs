//! Serialize a Rust data structure into JSON data

use std::{error, fmt};

use serde::ser;

use std::vec::Vec;

use self::map::SerializeMap;
use self::seq::SerializeSeq;
use self::struct_::SerializeStruct;

mod map;
mod seq;
mod struct_;

/// Serialization result
pub type Result<T> = ::core::result::Result<T, Error>;

/// This type represents all possible errors that can occur when serializing JSON data
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Buffer is full
    BufferFull,

    /// Custom error message from serde
    Custom(String),
}

impl From<()> for Error {
    fn from(_: ()) -> Error {
        Error::BufferFull
    }
}

impl From<u8> for Error {
    fn from(_: u8) -> Error {
        Error::BufferFull
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "(use display)"
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BufferFull => write!(f, "Buffer is full"),
            Error::Custom(msg) => write!(f, "{}", &msg),
        }
    }
}

/// Serializer implements serde::ser::Serializer and allows us to serialize a
/// serde struct into JSON
pub struct Serializer {
    buf: Vec<u8>,
}

/// Number of bytes reserved by default for the output JSON
static INITIAL_CAPACITY: usize = 1024;

impl Serializer {
    fn new() -> Self {
        Serializer {
            buf: Vec::with_capacity(INITIAL_CAPACITY),
        }
    }
}

// NOTE(serialize_*signed) This is basically the numtoa implementation minus the lookup tables,
// which take 200+ bytes of ROM / Flash
macro_rules! serialize_unsigned {
    ($self:ident, $N:expr, $v:expr) => {{
        let mut buf = [0u8; $N];

        let mut v = $v;
        let mut i = $N - 1;
        loop {
            buf[i] = (v % 10) as u8 + b'0';
            v /= 10;

            if v == 0 {
                break;
            } else {
                i -= 1;
            }
        }

        $self.buf.extend_from_slice(&buf[i..]);
        Ok(())
    }};
}
// Export for use in map
pub(crate) use serialize_unsigned;

macro_rules! serialize_signed {
    ($self:ident, $N:expr, $v:expr, $ixx:ident, $uxx:ident) => {{
        let v = $v;
        let (signed, mut v) = if v == $ixx::min_value() {
            (true, $ixx::max_value() as $uxx + 1)
        } else if v < 0 {
            (true, -v as $uxx)
        } else {
            (false, v as $uxx)
        };

        let mut buf = [0u8; $N];
        let mut i = $N - 1;
        loop {
            buf[i] = (v % 10) as u8 + b'0';
            v /= 10;

            i -= 1;

            if v == 0 {
                break;
            }
        }

        if signed {
            buf[i] = b'-';
        } else {
            i += 1;
        }
        $self.buf.extend_from_slice(&buf[i..]);
        Ok(())
    }};
}
// Export for use in map
pub(crate) use serialize_signed;

/// Upper-case hex for value in 0..16, encoded as ASCII bytes
fn hex_4bit(c: u8) -> u8 {
    if c <= 9 {
        0x30 + c
    } else {
        0x41 + (c - 10)
    }
}

/// Upper-case hex for value in 0..256, encoded as ASCII bytes
fn hex(c: u8) -> (u8, u8) {
    (hex_4bit(c >> 4), hex_4bit(c & 0x0F))
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = SerializeSeq<'a>;
    type SerializeTuple = SerializeSeq<'a>;
    type SerializeTupleStruct = Unreachable;
    type SerializeTupleVariant = SerializeSeq<'a>;
    type SerializeMap = SerializeMap<'a>;
    type SerializeStruct = SerializeStruct<'a>;
    type SerializeStructVariant = SerializeStruct<'a>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        if v {
            self.buf.extend_from_slice(b"true");
        } else {
            self.buf.extend_from_slice(b"false");
        }
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        // -128
        serialize_signed!(self, 4, v, i8, u8)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        // -32768
        serialize_signed!(self, 6, v, i16, u16)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        // -2147483648
        serialize_signed!(self, 11, v, i32, u32)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        // -9223372036854775808
        serialize_signed!(self, 20, v, i64, u64)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok> {
        // -170141183460469231731687303715884105728
        self.buf.push(b'"');
        let res: Result<Self::Ok> = serialize_signed!(self, 40, v, i128, u128);
        res?;
        self.buf.push(b'"');
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        // 255
        serialize_unsigned!(self, 3, v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        // 65535
        serialize_unsigned!(self, 5, v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        // 4294967295
        serialize_unsigned!(self, 10, v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        // 18446744073709551615
        serialize_unsigned!(self, 20, v)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok> {
        // 340282366920938463463374607431768211455
        self.buf.push(b'"');
        let res: Result<Self::Ok> = serialize_unsigned!(self, 39, v);
        res?;
        self.buf.push(b'"');
        Ok(())
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.buf.push(b'"');

        // Do escaping according to "6. MUST represent all strings (including object member names) in
        // their minimal-length UTF-8 encoding": https://gibson042.github.io/canonicaljson-spec/
        //
        // We don't need to escape lone surrogates because surrogate pairs do not exist in valid UTF-8,
        // even if they can exist in JSON or JavaScript strings (UCS-2 based). As a result, lone surrogates
        // cannot exist in a Rust String. If they do, the bug is in the String constructor.
        // An excellent explanation is available at https://www.youtube.com/watch?v=HhIEDWmQS3w

        // Temporary storage for encoded a single char.
        // A char is up to 4 bytes long when encoded to UTF-8.
        let mut encoding_tmp = [0u8; 4];

        for c in v.chars() {
            match c {
                '\\' => {
                    self.buf.push(b'\\');
                    self.buf.push(b'\\');
                }
                '"' => {
                    self.buf.push(b'\\');
                    self.buf.push(b'"');
                }
                '\u{0008}' => {
                    self.buf.push(b'\\');
                    self.buf.push(b'b');
                }
                '\u{0009}' => {
                    self.buf.push(b'\\');
                    self.buf.push(b't');
                }
                '\u{000A}' => {
                    self.buf.push(b'\\');
                    self.buf.push(b'n');
                }
                '\u{000C}' => {
                    self.buf.push(b'\\');
                    self.buf.push(b'f');
                }
                '\u{000D}' => {
                    self.buf.push(b'\\');
                    self.buf.push(b'r');
                }
                '\u{0000}'..='\u{001F}' => {
                    self.buf.push(b'\\');
                    self.buf.push(b'u');
                    self.buf.push(b'0');
                    self.buf.push(b'0');
                    let (hex1, hex2) = hex(c as u8);
                    self.buf.push(hex1);
                    self.buf.push(hex2);
                }
                _ => {
                    if c.len_utf8() == 1 {
                        self.buf.push(c as u8);
                    } else {
                        let encoded = c.encode_utf8(&mut encoding_tmp as &mut [u8]);
                        self.buf.extend_from_slice(encoded.as_bytes());
                    }
                }
            }
        }

        self.buf.push(b'"');
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        unreachable!()
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.buf.extend_from_slice(b"null");
        Ok(())
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        // The unit type is a zero element tuple, so the consistent way to serialize this would be "[]".
        // However, for compatibility with serde_json we serialize to "null".
        self.buf.extend_from_slice(b"null");
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        // Unit struct is serialized to (serde_json compatible) "null"
        self.buf.extend_from_slice(b"null");
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        value.serialize(&mut *self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ser::Serialize,
    {
        self.buf.push(b'{');
        self.serialize_str(variant)?;
        self.buf.push(b':');
        value.serialize(&mut *self)?;
        self.buf.push(b'}');
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.buf.push(b'[');

        Ok(SerializeSeq::new(self))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(_len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        unreachable!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.buf.push(b'{');
        self.serialize_str(variant)?;
        self.buf.push(b':');
        self.serialize_tuple(len)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.buf.push(b'{');
        Ok(SerializeMap::new(self))
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        self.buf.push(b'{');

        Ok(SerializeStruct::new(self))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.buf.push(b'{');
        self.serialize_str(variant)?;
        self.buf.push(b':');
        self.serialize_struct(name, len)
    }
}

/// Serializes the given data structure as a string of JSON text
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ser::Serialize + ?Sized,
{
    let mut ser = Serializer::new();
    value.serialize(&mut ser)?;
    Ok(unsafe { String::from_utf8_unchecked(ser.buf) })
}

/// Serializes the given data structure as a JSON byte vector
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ser::Serialize + ?Sized,
{
    let mut ser = Serializer::new();
    value.serialize(&mut ser)?;
    Ok(ser.buf)
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

/// Unreachable is a placeholder for features that are not supported
/// (and should be unreachable, unless you use unsupported serde flags)
pub enum Unreachable {}

impl ser::SerializeTupleStruct for Unreachable {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeTupleVariant for Unreachable {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _value: &T) -> Result<()> {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeMap for Unreachable {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, _key: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T: ?Sized>(&mut self, _value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

impl ser::SerializeStructVariant for Unreachable {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok> {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {

    use super::to_string;
    use serde::{Serialize, Serializer};
    use serde_derive::{Deserialize, Serialize};

    #[test]
    fn bool() {
        assert_eq!(to_string(&true).unwrap(), "true");
        assert_eq!(to_string(&false).unwrap(), "false");
    }

    #[test]
    fn number() {
        assert_eq!(to_string::<u8>(&0).unwrap(), "0");
        assert_eq!(to_string::<u8>(&1).unwrap(), "1");
        assert_eq!(to_string::<u8>(&std::u8::MAX).unwrap(), "255");

        assert_eq!(to_string::<i8>(&0).unwrap(), "0");
        assert_eq!(to_string::<i8>(&1).unwrap(), "1");
        assert_eq!(to_string::<i8>(&127).unwrap(), "127");
        assert_eq!(to_string::<i8>(&-1).unwrap(), "-1");
        assert_eq!(to_string::<i8>(&std::i8::MIN).unwrap(), "-128");

        assert_eq!(to_string::<u16>(&0).unwrap(), "0");
        assert_eq!(to_string::<u16>(&1).unwrap(), "1");
        assert_eq!(to_string::<u16>(&550).unwrap(), "550");
        assert_eq!(to_string::<u16>(&std::u16::MAX).unwrap(), "65535");

        assert_eq!(to_string::<i16>(&0).unwrap(), "0");
        assert_eq!(to_string::<i16>(&1).unwrap(), "1");
        assert_eq!(to_string::<i16>(&550).unwrap(), "550");
        assert_eq!(to_string::<i16>(&std::i16::MAX).unwrap(), "32767");
        assert_eq!(to_string::<i16>(&-1).unwrap(), "-1");
        assert_eq!(to_string::<i16>(&std::i16::MIN).unwrap(), "-32768");

        assert_eq!(to_string::<u32>(&0).unwrap(), "0");
        assert_eq!(to_string::<u32>(&1).unwrap(), "1");
        assert_eq!(to_string::<u32>(&456789).unwrap(), "456789");
        assert_eq!(to_string::<u32>(&std::u32::MAX).unwrap(), "4294967295");

        assert_eq!(to_string::<i32>(&0).unwrap(), "0");
        assert_eq!(to_string::<i32>(&1).unwrap(), "1");
        assert_eq!(to_string::<i32>(&456789).unwrap(), "456789");
        assert_eq!(to_string::<i32>(&std::i32::MAX).unwrap(), "2147483647");
        assert_eq!(to_string::<i32>(&-1).unwrap(), "-1");
        assert_eq!(to_string::<i32>(&std::i32::MIN).unwrap(), "-2147483648");

        assert_eq!(to_string::<u64>(&0).unwrap(), "0");
        assert_eq!(to_string::<u64>(&1).unwrap(), "1");
        assert_eq!(to_string::<u64>(&456789).unwrap(), "456789");
        assert_eq!(to_string::<u64>(&4294967295).unwrap(), "4294967295");
        assert_eq!(to_string::<u64>(&4294967296).unwrap(), "4294967296");
        assert_eq!(
            to_string::<u64>(&9007199254740991).unwrap(),
            "9007199254740991"
        ); // Number.MAX_SAFE_INTEGER
        assert_eq!(
            to_string::<u64>(&9007199254740992).unwrap(),
            "9007199254740992"
        ); // Number.MAX_SAFE_INTEGER+1
        assert_eq!(
            to_string::<u64>(&std::u64::MAX).unwrap(),
            "18446744073709551615"
        );

        assert_eq!(to_string::<i64>(&0).unwrap(), "0");
        assert_eq!(to_string::<i64>(&1).unwrap(), "1");
        assert_eq!(to_string::<i64>(&456789).unwrap(), "456789");
        assert_eq!(to_string::<i64>(&4294967295).unwrap(), "4294967295");
        assert_eq!(to_string::<i64>(&4294967296).unwrap(), "4294967296");
        assert_eq!(
            to_string::<i64>(&9007199254740991).unwrap(),
            "9007199254740991"
        ); // Number.MAX_SAFE_INTEGER
        assert_eq!(
            to_string::<i64>(&9007199254740992).unwrap(),
            "9007199254740992"
        ); // Number.MAX_SAFE_INTEGER+1
        assert_eq!(
            to_string::<i64>(&std::i64::MAX).unwrap(),
            "9223372036854775807"
        );
        assert_eq!(to_string::<i64>(&-1).unwrap(), "-1");
        assert_eq!(
            to_string::<i64>(&std::i64::MIN).unwrap(),
            "-9223372036854775808"
        );

        assert_eq!(to_string::<u128>(&0).unwrap(), r#""0""#);
        assert_eq!(to_string::<u128>(&1).unwrap(), r#""1""#);
        assert_eq!(to_string::<u128>(&456789).unwrap(), r#""456789""#);
        assert_eq!(to_string::<u128>(&4294967295).unwrap(), r#""4294967295""#);
        assert_eq!(to_string::<u128>(&4294967296).unwrap(), r#""4294967296""#);
        assert_eq!(
            to_string::<u128>(&9007199254740991).unwrap(),
            r#""9007199254740991""#
        ); // Number.MAX_SAFE_INTEGER
        assert_eq!(
            to_string::<u128>(&9007199254740992).unwrap(),
            r#""9007199254740992""#
        ); // Number.MAX_SAFE_INTEGER+1
        assert_eq!(
            to_string::<u128>(&9223372036854775807).unwrap(),
            r#""9223372036854775807""#
        );
        assert_eq!(
            to_string::<u128>(&9223372036854775808).unwrap(),
            r#""9223372036854775808""#
        );
        assert_eq!(
            to_string::<u128>(&std::u128::MAX).unwrap(),
            r#""340282366920938463463374607431768211455""#
        );

        assert_eq!(to_string::<i128>(&0).unwrap(), r#""0""#);
        assert_eq!(to_string::<i128>(&1).unwrap(), r#""1""#);
        assert_eq!(to_string::<i128>(&456789).unwrap(), r#""456789""#);
        assert_eq!(to_string::<i128>(&4294967295).unwrap(), r#""4294967295""#);
        assert_eq!(to_string::<i128>(&4294967296).unwrap(), r#""4294967296""#);
        assert_eq!(
            to_string::<i128>(&9007199254740991).unwrap(),
            r#""9007199254740991""#
        ); // Number.MAX_SAFE_INTEGER
        assert_eq!(
            to_string::<i128>(&9007199254740992).unwrap(),
            r#""9007199254740992""#
        ); // Number.MAX_SAFE_INTEGER+1
        assert_eq!(
            to_string::<i128>(&9223372036854775807).unwrap(),
            r#""9223372036854775807""#
        );
        assert_eq!(
            to_string::<i128>(&9223372036854775808).unwrap(),
            r#""9223372036854775808""#
        );
        assert_eq!(
            to_string::<i128>(&std::i128::MAX).unwrap(),
            r#""170141183460469231731687303715884105727""#
        );
        assert_eq!(to_string::<i128>(&-1).unwrap(), r#""-1""#);
        assert_eq!(
            to_string::<i128>(&std::i128::MIN).unwrap(),
            r#""-170141183460469231731687303715884105728""#
        );
    }

    #[test]
    fn array() {
        assert_eq!(to_string::<[u8]>(&[]).unwrap(), "[]");
        assert_eq!(to_string(&[0, 1, 2]).unwrap(), "[0,1,2]");
    }

    #[test]
    fn tuple() {
        type Pair = (i64, i64);
        type Wrapped = (i64,); // Comma differentiates one element tuple from a primary type surrounded by parentheses
        type Unit = ();

        let pair: Pair = (1, 2);
        assert_eq!(to_string(&pair).unwrap(), "[1,2]");
        assert_eq!(
            to_string(&pair).unwrap(),
            serde_json::to_string(&pair).unwrap()
        );

        let wrapped: Wrapped = (5,);
        assert_eq!(to_string(&wrapped).unwrap(), "[5]");
        assert_eq!(
            to_string(&wrapped).unwrap(),
            serde_json::to_string(&wrapped).unwrap()
        );

        #[allow(clippy::let_unit_value)]
        let unit: Unit = ();
        assert_eq!(to_string(&unit).unwrap(), "null");
        assert_eq!(
            to_string(&unit).unwrap(),
            serde_json::to_string(&unit).unwrap()
        );

        type BigPair = (u128, u128);

        let pair: BigPair = (std::u128::MAX, std::u128::MAX);

        assert_eq!(
            to_string(&pair).unwrap(),
            r#"["340282366920938463463374607431768211455","340282366920938463463374607431768211455"]"#
        );
    }

    #[test]
    fn enum_variants_unit_like() {
        #[allow(dead_code)]
        #[derive(Serialize)]
        enum Op {
            Enter,
            Exit,
        }
        assert_eq!(to_string(&Op::Exit).unwrap(), r#""Exit""#);
        assert_eq!(
            to_string(&Op::Exit).unwrap(),
            serde_json::to_string(&Op::Exit).unwrap()
        );

        // Numeric values are ignored 🤷
        #[derive(Serialize)]
        enum Order {
            Unordered = 1,
            Ordered = 42,
        }
        assert_eq!(to_string(&Order::Unordered).unwrap(), r#""Unordered""#);
        assert_eq!(
            to_string(&Order::Unordered).unwrap(),
            serde_json::to_string(&Order::Unordered).unwrap()
        );
        assert_eq!(to_string(&Order::Ordered).unwrap(), r#""Ordered""#);
        assert_eq!(
            to_string(&Order::Ordered).unwrap(),
            serde_json::to_string(&Order::Ordered).unwrap()
        );
    }

    #[test]
    fn enum_variants_tuple_like_structs() {
        #[derive(Serialize)]
        enum Op {
            Exit(),
            Square(i32),
            Add(i64, i64),
        }
        assert_eq!(to_string(&Op::Exit()).unwrap(), r#"{"Exit":[]}"#);
        assert_eq!(
            to_string(&Op::Exit()).unwrap(),
            serde_json::to_string(&Op::Exit()).unwrap()
        );
        assert_eq!(to_string(&Op::Square(2)).unwrap(), r#"{"Square":2}"#);
        assert_eq!(
            to_string(&Op::Square(2)).unwrap(),
            serde_json::to_string(&Op::Square(2)).unwrap()
        );
        assert_eq!(to_string(&Op::Add(3, 4)).unwrap(), r#"{"Add":[3,4]}"#);
        assert_eq!(
            to_string(&Op::Add(3, 4)).unwrap(),
            serde_json::to_string(&Op::Add(3, 4)).unwrap()
        );
    }

    #[test]
    fn enum_variants_c_like_structs() {
        #[derive(Serialize)]
        enum Op {
            Exit {},
            Square { input: i32 },
            Add { a: i64, b: i64 },
        }
        assert_eq!(to_string(&Op::Exit {}).unwrap(), r#"{"Exit":{}}"#);
        assert_eq!(
            to_string(&Op::Exit {}).unwrap(),
            serde_json::to_string(&Op::Exit {}).unwrap()
        );
        assert_eq!(
            to_string(&Op::Square { input: 2 }).unwrap(),
            r#"{"Square":{"input":2}}"#
        );
        assert_eq!(
            to_string(&Op::Square { input: 2 }).unwrap(),
            serde_json::to_string(&Op::Square { input: 2 }).unwrap()
        );
        assert_eq!(
            to_string(&Op::Add { a: 3, b: 4 }).unwrap(),
            r#"{"Add":{"a":3,"b":4}}"#
        );
        assert_eq!(
            to_string(&Op::Add { a: 3, b: 4 }).unwrap(),
            serde_json::to_string(&Op::Add { a: 3, b: 4 }).unwrap()
        );
    }

    #[test]
    fn enum_mixed() {
        #[derive(Serialize)]
        enum Animal {
            Ant,
            #[serde(rename = "kitty")]
            Cat,
            Dog(),
            Horse {},
            Zebra {
                height: u32,
            },
        }
        assert_eq!(to_string(&Animal::Ant).unwrap(), r#""Ant""#);
        assert_eq!(to_string(&Animal::Cat).unwrap(), r#""kitty""#);
        assert_eq!(to_string(&Animal::Dog()).unwrap(), r#"{"Dog":[]}"#);
        assert_eq!(to_string(&Animal::Horse {}).unwrap(), r#"{"Horse":{}}"#);
        assert_eq!(
            to_string(&Animal::Zebra { height: 273 }).unwrap(),
            r#"{"Zebra":{"height":273}}"#
        );
    }

    #[test]
    fn str() {
        assert_eq!(to_string("hello").unwrap(), r#""hello""#);
        assert_eq!(to_string("").unwrap(), r#""""#);

        // Characters unescaped if possible
        assert_eq!(to_string("ä").unwrap(), r#""ä""#);
        assert_eq!(to_string("৬").unwrap(), r#""৬""#);
        assert_eq!(to_string("\u{A0}").unwrap(), r#"" ""#); // non-breaking space
        assert_eq!(to_string("ℝ").unwrap(), r#""ℝ""#); // 3 byte character
        assert_eq!(to_string("💣").unwrap(), r#""💣""#); // 4 byte character

        // " and \ must be escaped
        assert_eq!(to_string("foo\"bar").unwrap(), r#""foo\"bar""#);
        assert_eq!(to_string("foo\\bar").unwrap(), r#""foo\\bar""#);

        // \b, \t, \n, \f, \r must be escaped in their two-character escaping
        assert_eq!(to_string(" \u{0008} ").unwrap(), r#"" \b ""#);
        assert_eq!(to_string(" \u{0009} ").unwrap(), r#"" \t ""#);
        assert_eq!(to_string(" \u{000A} ").unwrap(), r#"" \n ""#);
        assert_eq!(to_string(" \u{000C} ").unwrap(), r#"" \f ""#);
        assert_eq!(to_string(" \u{000D} ").unwrap(), r#"" \r ""#);

        // U+0000 through U+001F is escaped using six-character \u00xx uppercase hexadecimal escape sequences
        assert_eq!(to_string(" \u{0000} ").unwrap(), r#"" \u0000 ""#);
        assert_eq!(to_string(" \u{0001} ").unwrap(), r#"" \u0001 ""#);
        assert_eq!(to_string(" \u{0007} ").unwrap(), r#"" \u0007 ""#);
        assert_eq!(to_string(" \u{000e} ").unwrap(), r#"" \u000E ""#);
        assert_eq!(to_string(" \u{001D} ").unwrap(), r#"" \u001D ""#);
        assert_eq!(to_string(" \u{001f} ").unwrap(), r#"" \u001F ""#);
    }

    #[test]
    fn collect_str_can_be_used_in_custom_seralize_impl() {
        struct SpecialType {
            count: u32,
            on: bool,
        }

        impl Serialize for SpecialType {
            // A custom Serialize implementation for SpecialType. SpecialType is giving us the chance to use
            // an efficient collect_str implementation that is better than allocating the String and running
            // serialize_str on it.
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.collect_str(&format_args!("{}-{}", self.count, self.on))
            }
        }

        let value = SpecialType {
            count: 123,
            on: false,
        };
        assert_eq!(to_string(&value).unwrap(), r#""123-false""#);
    }

    #[test]
    fn newtype() {
        #[derive(Serialize)]
        struct Address(String);
        #[derive(Serialize)]
        struct CommentId(u32);

        assert_eq!(
            to_string(&Address("home".to_string())).unwrap(),
            r#""home""#
        );
        assert_eq!(to_string(&CommentId(42)).unwrap(), r#"42"#);
    }

    #[test]
    fn struct_bool() {
        #[derive(Serialize)]
        struct Led {
            led: bool,
        }

        assert_eq!(to_string(&Led { led: true }).unwrap(), r#"{"led":true}"#);
    }

    #[test]
    fn struct_i8() {
        #[derive(Serialize)]
        struct Temperature {
            temperature: i8,
        }

        assert_eq!(
            to_string(&Temperature { temperature: 127 }).unwrap(),
            r#"{"temperature":127}"#
        );

        assert_eq!(
            to_string(&Temperature { temperature: 20 }).unwrap(),
            r#"{"temperature":20}"#
        );

        assert_eq!(
            to_string(&Temperature { temperature: -17 }).unwrap(),
            r#"{"temperature":-17}"#
        );

        assert_eq!(
            to_string(&Temperature { temperature: -128 }).unwrap(),
            r#"{"temperature":-128}"#
        );
    }

    #[test]
    fn struct_option() {
        #[derive(Serialize)]
        struct Property<'a> {
            description: Option<&'a str>,
        }

        assert_eq!(
            to_string(&Property {
                description: Some("An ambient temperature sensor"),
            })
            .unwrap(),
            r#"{"description":"An ambient temperature sensor"}"#
        );

        // XXX Ideally this should produce "{}"
        assert_eq!(
            to_string(&Property { description: None }).unwrap(),
            r#"{"description":null}"#
        );
    }

    #[test]
    fn struct_u8() {
        #[derive(Serialize)]
        struct Temperature {
            temperature: u8,
        }

        assert_eq!(
            to_string(&Temperature { temperature: 20 }).unwrap(),
            r#"{"temperature":20}"#
        );
    }

    #[test]
    fn struct_() {
        #[derive(Serialize)]
        struct Nothing;

        assert_eq!(to_string(&Nothing).unwrap(), r#"null"#);
        assert_eq!(
            to_string(&Nothing).unwrap(),
            serde_json::to_string(&Nothing).unwrap()
        );

        #[derive(Serialize)]
        struct Empty {}

        assert_eq!(to_string(&Empty {}).unwrap(), r#"{}"#);
        assert_eq!(
            to_string(&Empty {}).unwrap(),
            serde_json::to_string(&Empty {}).unwrap()
        );

        #[derive(Serialize)]
        struct Tuple {
            a: bool,
            b: bool,
        }

        assert_eq!(
            to_string(&Tuple { a: true, b: false }).unwrap(),
            r#"{"a":true,"b":false}"#
        );
    }

    #[test]
    fn struct_with_flatten() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Pagination {
            limit: u64,
            offset: u64,
            total: u64,
        }

        #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
        struct Users {
            users: Vec<String>,

            #[serde(flatten)]
            pagination: Pagination,
        }

        let users = Users {
            users: vec!["joe".to_string(), "alice".to_string()],
            pagination: Pagination {
                offset: 100,
                limit: 20,
                total: 102,
            },
        };

        assert_eq!(
            to_string(&users).unwrap(),
            r#"{"users":["joe","alice"],"limit":20,"offset":100,"total":102}"#
        );
        assert_eq!(
            to_string(&users).unwrap(),
            serde_json::to_string(&users).unwrap(),
            "serialization must match serde_json implementation"
        );
    }

    #[test]
    fn btree_map() {
        use std::collections::BTreeMap;

        // empty map
        assert_eq!(to_string(&BTreeMap::<(), ()>::new()).unwrap(), r#"{}"#);

        // One element with unit type
        let mut map = BTreeMap::<&str, ()>::new();
        map.insert("set_element", ());
        assert_eq!(to_string(&map).unwrap(), r#"{"set_element":null}"#);

        let mut two_values = BTreeMap::new();
        two_values.insert("my_name", "joseph");
        two_values.insert("her_name", "aline");
        assert_eq!(
            to_string(&two_values).unwrap(),
            r#"{"her_name":"aline","my_name":"joseph"}"#
        );

        let mut nested_map = BTreeMap::new();
        nested_map.insert("two_entries", two_values.clone());

        two_values.remove("my_name");
        nested_map.insert("one_entry", two_values);
        assert_eq!(
            to_string(&nested_map).unwrap(),
            r#"{"one_entry":{"her_name":"aline"},"two_entries":{"her_name":"aline","my_name":"joseph"}}"#
        );
    }

    #[test]
    fn hash_map() {
        use std::collections::HashMap;

        // empty map
        assert_eq!(to_string(&HashMap::<(), ()>::new()).unwrap(), r#"{}"#);

        // One element
        let mut map = HashMap::new();
        map.insert("my_age", 28);
        assert_eq!(to_string(&map).unwrap(), r#"{"my_age":28}"#);

        #[derive(Debug, Serialize, PartialEq, Eq, Hash)]
        pub struct NewType(String);

        // New type wrappers around String types work as keys
        let mut map = HashMap::new();
        map.insert(NewType(String::from("my_age")), 44);
        assert_eq!(to_string(&map).unwrap(), r#"{"my_age":44}"#);

        #[derive(Debug, Serialize, PartialEq, Eq, Hash)]
        #[serde(rename_all = "lowercase")]
        pub enum MyResult {
            Err,
        }

        // Unit variants are also valid keys
        let mut map = HashMap::new();
        map.insert(MyResult::Err, 404);
        assert_eq!(to_string(&map).unwrap(), r#"{"err":404}"#);

        // HashMap does not have deterministic iteration order (except in the Wasm target).
        // So the two element map is serialized as one of two options.
        let mut two_values = HashMap::new();
        two_values.insert("my_name", "joseph");
        two_values.insert("her_name", "aline");
        let serialized = to_string(&two_values).unwrap();
        assert!(
            serialized == r#"{"her_name":"aline","my_name":"joseph"}"#
                || serialized == r#"{"my_name":"joseph","her_name":"aline"}"#
        );
    }

    #[test]
    fn map_serialization_matches_json_serde() {
        use std::collections::BTreeMap;

        fn ser_actual<T: serde::Serialize + ?Sized>(value: &T) -> String {
            to_string(value).unwrap()
        }

        fn ser_expected<T: serde::Serialize + ?Sized>(value: &T) -> String {
            serde_json::to_string(value).unwrap()
        }

        let map = BTreeMap::<(), ()>::new();
        assert_eq!(ser_actual(&map), ser_expected(&map));

        let mut two_values = BTreeMap::new();
        two_values.insert("my_name", "joseph");
        two_values.insert("her_name", "aline");
        assert_eq!(ser_actual(&two_values), ser_expected(&two_values));

        let mut nested_map = BTreeMap::new();
        nested_map.insert("two_entries", two_values.clone());
        two_values.remove("my_name");
        nested_map.insert("one_entry", two_values);
        assert_eq!(ser_actual(&nested_map), ser_expected(&nested_map));

        // One element with unit type
        let mut map = BTreeMap::<&str, ()>::new();
        map.insert("set_element", ());
        assert_eq!(ser_actual(&map), ser_expected(&map));

        // numeric keys
        let mut map = BTreeMap::new();
        map.insert(10i8, "my_age");
        assert_eq!(ser_actual(&map), ser_expected(&map));

        // numeric values
        let mut scores = BTreeMap::new();
        scores.insert("player A", 1234212);
        assert_eq!(ser_actual(&scores), ser_expected(&scores));
    }

    #[test]
    fn number_key() {
        use std::collections::HashMap;

        // i8 key
        let mut map = HashMap::new();
        map.insert(10i8, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // i16 key
        let mut map = HashMap::new();
        map.insert(10i16, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // i32 key
        let mut map = HashMap::new();
        map.insert(10i32, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // i64 key
        let mut map = HashMap::new();
        map.insert(10i64, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // i128 key
        let mut map = HashMap::new();
        map.insert(10i128, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // u8 key
        let mut map = HashMap::new();
        map.insert(10u8, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // u16 key
        let mut map = HashMap::new();
        map.insert(10u16, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // u32 key
        let mut map = HashMap::new();
        map.insert(10u32, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // u64 key
        let mut map = HashMap::new();
        map.insert(10u64, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);

        // u128 key
        let mut map = HashMap::new();
        map.insert(10u128, "my_age");
        assert_eq!(to_string(&map).unwrap(), r#"{"10":"my_age"}"#);
    }

    #[test]
    fn invalid_json_key() {
        use crate::ser::map::key_must_be_a_string;
        use std::collections::HashMap;

        #[derive(Debug, Serialize, PartialEq, Eq, Hash)]
        #[serde(rename_all = "lowercase")]
        pub enum MyResult {
            Unit(()),
            Ok(Response),
        }
        #[derive(Debug, Serialize, PartialEq, Eq, Hash)]
        pub struct Response {
            pub log: Option<String>,
            pub count: i64,
            pub list: Vec<u32>,
        }

        // unit enum
        let mut map = HashMap::new();
        map.insert(MyResult::Unit(()), "my_age");
        assert_eq!(
            to_string(&map).unwrap_err().to_string(),
            key_must_be_a_string().to_string()
        );

        // struct enum
        let mut map = HashMap::new();
        map.insert(
            MyResult::Ok(Response {
                log: None,
                count: 1,
                list: vec![6],
            }),
            "my_age",
        );
        assert_eq!(
            to_string(&map).unwrap_err().to_string(),
            key_must_be_a_string().to_string()
        );

        // Struct
        let mut map = HashMap::new();
        map.insert(
            Response {
                log: None,
                count: 1,
                list: vec![6],
            },
            "my_age",
        );
        assert_eq!(
            to_string(&map).unwrap_err().to_string(),
            key_must_be_a_string().to_string()
        );
    }

    #[test]
    fn serialize_embedded_enum() {
        use serde_derive::Deserialize;

        #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
        #[serde(rename_all = "lowercase")]
        pub enum MyResult {
            Unit(()),
            Ok(Response),
            Err(String),
        }

        #[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
        pub struct Response {
            pub log: Option<String>,
            pub count: i64,
            pub list: Vec<u32>,
        }

        let err_input = MyResult::Err("some error".to_string());
        let json = to_string(&err_input).expect("encode err enum");
        assert_eq!(json, r#"{"err":"some error"}"#.to_string());
        let loaded = crate::from_str(&json).expect("re-load err enum");
        assert_eq!(err_input, loaded);

        let unit = MyResult::Unit(());
        let json = to_string(&unit).expect("encode unit enum");
        assert_eq!(json, r#"{"unit":null}"#.to_string());
        let loaded = crate::from_str(&json).expect("re-load unit enum");
        assert_eq!(unit, loaded);

        let empty_list = MyResult::Ok(Response {
            log: Some("log message".to_string()),
            count: 137,
            list: Vec::new(),
        });
        let json = to_string(&empty_list).expect("encode ok enum");
        assert_eq!(
            json,
            r#"{"ok":{"log":"log message","count":137,"list":[]}}"#.to_string()
        );
        let loaded = crate::from_str(&json).expect("re-load ok enum");
        assert_eq!(empty_list, loaded);

        let full_list = MyResult::Ok(Response {
            log: None,
            count: 137,
            list: vec![18u32, 34, 12],
        });
        let json = to_string(&full_list).expect("encode ok enum");
        assert_eq!(
            json,
            r#"{"ok":{"log":null,"count":137,"list":[18,34,12]}}"#.to_string()
        );
        let loaded = crate::from_str(&json).expect("re-load ok enum");
        assert_eq!(full_list, loaded);
    }
}
