//! This module contains `Impossible` serializer and its implementations.

use lib::*;

use ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};

/// Helper type for implementing a `Serializer` that does not support
/// serializing one of the compound types.
///
/// This type cannot be instantiated, but implements every one of the traits
/// corresponding to the [`Serializer`] compound types: [`SerializeSeq`],
/// [`SerializeTuple`], [`SerializeTupleStruct`], [`SerializeTupleVariant`],
/// [`SerializeMap`], [`SerializeStruct`], and [`SerializeStructVariant`].
///
/// ```edition2018
/// # use serde::ser::{Serializer, Impossible};
/// # use serde::__private::doc::Error;
/// #
/// # struct MySerializer;
/// #
/// impl Serializer for MySerializer {
///     type Ok = ();
///     type Error = Error;
///
///     type SerializeSeq = Impossible<(), Error>;
///     /* other associated types */
///
///     /// This data format does not support serializing sequences.
///     fn serialize_seq(self,
///                      len: Option<usize>)
///                      -> Result<Self::SerializeSeq, Error> {
///         // Given Impossible cannot be instantiated, the only
///         // thing we can do here is to return an error.
/// #         stringify! {
///         Err(...)
/// #         };
/// #         unimplemented!()
///     }
///
///     /* other Serializer methods */
/// #     serde::__serialize_unimplemented! {
/// #         bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str bytes none some
/// #         unit unit_struct unit_variant newtype_struct newtype_variant
/// #         tuple tuple_struct tuple_variant map struct struct_variant
/// #     }
/// }
/// ```
///
/// [`Serializer`]: trait.Serializer.html
/// [`SerializeSeq`]: trait.SerializeSeq.html
/// [`SerializeTuple`]: trait.SerializeTuple.html
/// [`SerializeTupleStruct`]: trait.SerializeTupleStruct.html
/// [`SerializeTupleVariant`]: trait.SerializeTupleVariant.html
/// [`SerializeMap`]: trait.SerializeMap.html
/// [`SerializeStruct`]: trait.SerializeStruct.html
/// [`SerializeStructVariant`]: trait.SerializeStructVariant.html
pub struct Impossible<Ok, Error> {
    void: Void,
    ok: PhantomData<Ok>,
    error: PhantomData<Error>,
}

enum Void {}

impl<Ok, Error> SerializeSeq for Impossible<Ok, Error>
where
    Error: ser::Error,
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = value;
        match self.void {}
    }

    fn end(self) -> Result<Ok, Error> {
        match self.void {}
    }
}

impl<Ok, Error> SerializeTuple for Impossible<Ok, Error>
where
    Error: ser::Error,
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = value;
        match self.void {}
    }

    fn end(self) -> Result<Ok, Error> {
        match self.void {}
    }
}

impl<Ok, Error> SerializeTupleStruct for Impossible<Ok, Error>
where
    Error: ser::Error,
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = value;
        match self.void {}
    }

    fn end(self) -> Result<Ok, Error> {
        match self.void {}
    }
}

impl<Ok, Error> SerializeTupleVariant for Impossible<Ok, Error>
where
    Error: ser::Error,
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = value;
        match self.void {}
    }

    fn end(self) -> Result<Ok, Error> {
        match self.void {}
    }
}

impl<Ok, Error> SerializeMap for Impossible<Ok, Error>
where
    Error: ser::Error,
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = key;
        match self.void {}
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = value;
        match self.void {}
    }

    fn end(self) -> Result<Ok, Error> {
        match self.void {}
    }
}

impl<Ok, Error> SerializeStruct for Impossible<Ok, Error>
where
    Error: ser::Error,
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = key;
        let _ = value;
        match self.void {}
    }

    fn end(self) -> Result<Ok, Error> {
        match self.void {}
    }
}

impl<Ok, Error> SerializeStructVariant for Impossible<Ok, Error>
where
    Error: ser::Error,
{
    type Ok = Ok;
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        let _ = key;
        let _ = value;
        match self.void {}
    }

    fn end(self) -> Result<Ok, Error> {
        match self.void {}
    }
}
