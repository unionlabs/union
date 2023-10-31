use core::convert::TryFrom;
use core::hash::BuildHasher;
use core::marker::PhantomData;

use crate::maybestd::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, LinkedList, VecDeque},
    io::{ErrorKind, Result, Write},
    string::String,
    vec::Vec,
};

#[cfg(feature = "rc")]
use crate::maybestd::{rc::Rc, sync::Arc};

pub(crate) mod helpers;

const DEFAULT_SERIALIZER_CAPACITY: usize = 1024;

/// A data-structure that can be serialized into binary format by NBOR.
///
/// ```
/// use borsh::BorshSerialize;
///
/// #[derive(BorshSerialize)]
/// struct MyBorshSerializableStruct {
///     value: String,
/// }
///
/// let x = MyBorshSerializableStruct { value: "hello".to_owned() };
/// let mut buffer: Vec<u8> = Vec::new();
/// x.serialize(&mut buffer).unwrap();
/// let single_serialized_buffer_len = buffer.len();
///
/// x.serialize(&mut buffer).unwrap();
/// assert_eq!(buffer.len(), single_serialized_buffer_len * 2);
///
/// let mut buffer: Vec<u8> = vec![0; 1024 + single_serialized_buffer_len];
/// let mut buffer_slice_enough_for_the_data = &mut buffer[1024..1024 + single_serialized_buffer_len];
/// x.serialize(&mut buffer_slice_enough_for_the_data).unwrap();
/// ```
pub trait BorshSerialize {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()>;

    /// Serialize this instance into a vector of bytes.
    fn try_to_vec(&self) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(DEFAULT_SERIALIZER_CAPACITY);
        self.serialize(&mut result)?;
        Ok(result)
    }

    #[inline]
    #[doc(hidden)]
    fn u8_slice(slice: &[Self]) -> Option<&[u8]>
    where
        Self: Sized,
    {
        let _ = slice;
        None
    }
}

impl BorshSerialize for u8 {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(core::slice::from_ref(self))
    }

    #[inline]
    fn u8_slice(slice: &[Self]) -> Option<&[u8]> {
        Some(slice)
    }
}

macro_rules! impl_for_integer {
    ($type: ident) => {
        impl BorshSerialize for $type {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                let bytes = self.to_le_bytes();
                writer.write_all(&bytes)
            }
        }
    };
}

impl_for_integer!(i8);
impl_for_integer!(i16);
impl_for_integer!(i32);
impl_for_integer!(i64);
impl_for_integer!(i128);
impl_for_integer!(u16);
impl_for_integer!(u32);
impl_for_integer!(u64);
impl_for_integer!(u128);

macro_rules! impl_for_nonzero_integer {
    ($type: ty) => {
        impl BorshSerialize for $type {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                BorshSerialize::serialize(&self.get(), writer)
            }
        }
    };
}

impl_for_nonzero_integer!(core::num::NonZeroI8);
impl_for_nonzero_integer!(core::num::NonZeroI16);
impl_for_nonzero_integer!(core::num::NonZeroI32);
impl_for_nonzero_integer!(core::num::NonZeroI64);
impl_for_nonzero_integer!(core::num::NonZeroI128);
impl_for_nonzero_integer!(core::num::NonZeroU8);
impl_for_nonzero_integer!(core::num::NonZeroU16);
impl_for_nonzero_integer!(core::num::NonZeroU32);
impl_for_nonzero_integer!(core::num::NonZeroU64);
impl_for_nonzero_integer!(core::num::NonZeroU128);
impl_for_nonzero_integer!(core::num::NonZeroUsize);

impl BorshSerialize for isize {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        BorshSerialize::serialize(&(*self as i64), writer)
    }
}

impl BorshSerialize for usize {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        BorshSerialize::serialize(&(*self as u64), writer)
    }
}

// Note NaNs have a portability issue. Specifically, signalling NaNs on MIPS are quiet NaNs on x86,
// and vice-versa. We disallow NaNs to avoid this issue.
macro_rules! impl_for_float {
    ($type: ident) => {
        impl BorshSerialize for $type {
            #[inline]
            fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
                assert!(
                    !self.is_nan(),
                    "For portability reasons we do not allow to serialize NaNs."
                );
                writer.write_all(&self.to_bits().to_le_bytes())
            }
        }
    };
}

impl_for_float!(f32);
impl_for_float!(f64);

impl BorshSerialize for bool {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        (u8::from(*self)).serialize(writer)
    }
}

impl<T> BorshSerialize for core::ops::Range<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.start.serialize(writer)?;
        self.end.serialize(writer)
    }
}

impl<T> BorshSerialize for Option<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            None => 0u8.serialize(writer),
            Some(value) => {
                1u8.serialize(writer)?;
                value.serialize(writer)
            }
        }
    }
}

impl<T, E> BorshSerialize for core::result::Result<T, E>
where
    T: BorshSerialize,
    E: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        match self {
            Err(e) => {
                0u8.serialize(writer)?;
                e.serialize(writer)
            }
            Ok(v) => {
                1u8.serialize(writer)?;
                v.serialize(writer)
            }
        }
    }
}

impl BorshSerialize for str {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_bytes().serialize(writer)
    }
}

impl BorshSerialize for String {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_bytes().serialize(writer)
    }
}

/// Helper method that is used to serialize a slice of data (without the length marker).
#[inline]
fn serialize_slice<T: BorshSerialize, W: Write>(data: &[T], writer: &mut W) -> Result<()> {
    if let Some(u8_slice) = T::u8_slice(data) {
        writer.write_all(u8_slice)?;
    } else {
        for item in data {
            item.serialize(writer)?;
        }
    }
    Ok(())
}

impl<T> BorshSerialize for [T]
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(
            &(u32::try_from(self.len()).map_err(|_| ErrorKind::InvalidInput)?).to_le_bytes(),
        )?;
        serialize_slice(self, writer)
    }
}

impl<T: BorshSerialize + ?Sized> BorshSerialize for &T {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        (*self).serialize(writer)
    }
}

impl<T> BorshSerialize for Cow<'_, T>
where
    T: BorshSerialize + ToOwned + ?Sized,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_ref().serialize(writer)
    }
}

impl<T> BorshSerialize for Vec<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_slice().serialize(writer)
    }
}

impl<T> BorshSerialize for VecDeque<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(
            &(u32::try_from(self.len()).map_err(|_| ErrorKind::InvalidInput)?).to_le_bytes(),
        )?;
        let slices = self.as_slices();
        serialize_slice(slices.0, writer)?;
        serialize_slice(slices.1, writer)
    }
}

impl<T> BorshSerialize for LinkedList<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(
            &(u32::try_from(self.len()).map_err(|_| ErrorKind::InvalidInput)?).to_le_bytes(),
        )?;
        for item in self {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T> BorshSerialize for BinaryHeap<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        // It could have been just `self.as_slice().serialize(writer)`, but there is no
        // `as_slice()` method:
        // https://internals.rust-lang.org/t/should-i-add-as-slice-method-to-binaryheap/13816
        writer.write_all(
            &(u32::try_from(self.len()).map_err(|_| ErrorKind::InvalidInput)?).to_le_bytes(),
        )?;
        for item in self {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<K, V, H> BorshSerialize for HashMap<K, V, H>
where
    K: BorshSerialize + PartialOrd,
    V: BorshSerialize,
    H: BuildHasher,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut vec = self.iter().collect::<Vec<_>>();
        vec.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap());
        u32::try_from(vec.len())
            .map_err(|_| ErrorKind::InvalidInput)?
            .serialize(writer)?;
        for (key, value) in vec {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T, H> BorshSerialize for HashSet<T, H>
where
    T: BorshSerialize + PartialOrd,
    H: BuildHasher,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut vec = self.iter().collect::<Vec<_>>();
        vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
        u32::try_from(vec.len())
            .map_err(|_| ErrorKind::InvalidInput)?
            .serialize(writer)?;
        for item in vec {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<K, V> BorshSerialize for BTreeMap<K, V>
where
    K: BorshSerialize,
    V: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        // NOTE: BTreeMap iterates over the entries that are sorted by key, so the serialization
        // result will be consistent without a need to sort the entries as we do for HashMap
        // serialization.
        u32::try_from(self.len())
            .map_err(|_| ErrorKind::InvalidInput)?
            .serialize(writer)?;
        for (key, value) in self {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T> BorshSerialize for BTreeSet<T>
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        // NOTE: BTreeSet iterates over the items that are sorted, so the serialization result will
        // be consistent without a need to sort the entries as we do for HashSet serialization.
        u32::try_from(self.len())
            .map_err(|_| ErrorKind::InvalidInput)?
            .serialize(writer)?;
        for item in self {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
impl BorshSerialize for std::net::SocketAddr {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            std::net::SocketAddr::V4(ref addr) => {
                0u8.serialize(writer)?;
                addr.serialize(writer)
            }
            std::net::SocketAddr::V6(ref addr) => {
                1u8.serialize(writer)?;
                addr.serialize(writer)
            }
        }
    }
}

#[cfg(feature = "std")]
impl BorshSerialize for std::net::SocketAddrV4 {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.ip().serialize(writer)?;
        self.port().serialize(writer)
    }
}

#[cfg(feature = "std")]
impl BorshSerialize for std::net::SocketAddrV6 {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.ip().serialize(writer)?;
        self.port().serialize(writer)
    }
}

#[cfg(feature = "std")]
impl BorshSerialize for std::net::Ipv4Addr {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.octets())
    }
}

#[cfg(feature = "std")]
impl BorshSerialize for std::net::Ipv6Addr {
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.octets())
    }
}

impl<T: BorshSerialize + ?Sized> BorshSerialize for Box<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.as_ref().serialize(writer)
    }
}

impl<T, const N: usize> BorshSerialize for [T; N]
where
    T: BorshSerialize,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        if N == 0 {
            return Ok(());
        } else if let Some(u8_slice) = T::u8_slice(self) {
            writer.write_all(u8_slice)?;
        } else {
            for el in self.iter() {
                el.serialize(writer)?;
            }
        }
        Ok(())
    }
}

impl BorshSerialize for () {
    fn serialize<W: Write>(&self, _writer: &mut W) -> Result<()> {
        Ok(())
    }
}

macro_rules! impl_tuple {
    ($($idx:tt $name:ident)+) => {
      impl<$($name),+> BorshSerialize for ($($name,)+)
      where $($name: BorshSerialize,)+
      {
        #[inline]
        fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
            $(self.$idx.serialize(writer)?;)+
            Ok(())
        }
      }
    };
}

impl_tuple!(0 T0);
impl_tuple!(0 T0 1 T1);
impl_tuple!(0 T0 1 T1 2 T2);
impl_tuple!(0 T0 1 T1 2 T2 3 T3);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18);
impl_tuple!(0 T0 1 T1 2 T2 3 T3 4 T4 5 T5 6 T6 7 T7 8 T8 9 T9 10 T10 11 T11 12 T12 13 T13 14 T14 15 T15 16 T16 17 T17 18 T18 19 T19);

#[cfg(feature = "rc")]
impl<T: BorshSerialize + ?Sized> BorshSerialize for Rc<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        (**self).serialize(writer)
    }
}

#[cfg(feature = "rc")]
impl<T: BorshSerialize + ?Sized> BorshSerialize for Arc<T> {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        (**self).serialize(writer)
    }
}

impl<T: ?Sized> BorshSerialize for PhantomData<T> {
    fn serialize<W: Write>(&self, _: &mut W) -> Result<()> {
        Ok(())
    }
}
