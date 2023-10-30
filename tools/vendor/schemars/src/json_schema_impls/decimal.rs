use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

macro_rules! decimal_impl {
    ($type:ty) => {
        decimal_impl!($type => Number, "Number");
    };
    ($type:ty => $instance_type:ident, $name:expr) => {
        impl JsonSchema for $type {
            no_ref_schema!();

            fn schema_name() -> String {
                $name.to_owned()
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::$instance_type.into()),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

#[cfg(feature = "rust_decimal")]
decimal_impl!(rust_decimal::Decimal);
#[cfg(feature = "bigdecimal")]
decimal_impl!(bigdecimal::BigDecimal);
