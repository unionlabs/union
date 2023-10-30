use serde::ser;

use crate::ser::{Error, Result, Serializer};

pub struct SerializeSeq<'a> {
    ser: &'a mut Serializer,
    first: bool,
}

impl<'a> SerializeSeq<'a> {
    pub(crate) fn new(ser: &'a mut Serializer) -> Self {
        SerializeSeq { ser, first: true }
    }
}

impl<'a> ser::SerializeSeq for SerializeSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        if !self.first {
            self.ser.buf.push(b',');
        }
        self.first = false;

        value.serialize(&mut *self.ser)?;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.ser.buf.push(b']');
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for SerializeSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for SerializeSeq<'a> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        // close sequence
        self.ser.buf.push(b']');
        // close surrounding enum
        self.ser.buf.push(b'}');
        Ok(())
    }
}
