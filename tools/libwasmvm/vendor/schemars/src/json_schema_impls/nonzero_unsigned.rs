use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::num::*;

macro_rules! nonzero_unsigned_impl {
    ($type:ty => $primitive:ty) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                stringify!($type).to_owned()
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                let mut schema: SchemaObject = <$primitive>::json_schema(gen).into();
                schema.number().minimum = Some(1.0);
                schema.into()
            }
        }
    };
}

nonzero_unsigned_impl!(NonZeroU8 => u8);
nonzero_unsigned_impl!(NonZeroU16 => u16);
nonzero_unsigned_impl!(NonZeroU32 => u32);
nonzero_unsigned_impl!(NonZeroU64 => u64);
nonzero_unsigned_impl!(NonZeroU128 => u128);
nonzero_unsigned_impl!(NonZeroUsize => usize);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::schema_object_for;
    use pretty_assertions::assert_eq;

    #[test]
    fn schema_for_nonzero_u32() {
        let schema = schema_object_for::<NonZeroU32>();
        assert_eq!(schema.number.unwrap().minimum, Some(1.0));
        assert_eq!(schema.instance_type, Some(InstanceType::Integer.into()));
        assert_eq!(schema.format, Some("uint32".to_owned()));
    }
}
