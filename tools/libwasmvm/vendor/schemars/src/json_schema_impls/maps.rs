use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;

macro_rules! map_impl {
    ($($desc:tt)+) => {
        impl $($desc)+
        where
            V: JsonSchema,
        {
            no_ref_schema!();

            fn schema_name() -> String {
                format!("Map_of_{}", V::schema_name())
            }

            fn json_schema(gen: &mut SchemaGenerator) -> Schema {
                let subschema = gen.subschema_for::<V>();
                SchemaObject {
                    instance_type: Some(InstanceType::Object.into()),
                    object: Some(Box::new(ObjectValidation {
                        additional_properties: Some(Box::new(subschema)),
                        ..Default::default()
                    })),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

map_impl!(<K, V> JsonSchema for std::collections::BTreeMap<K, V>);
map_impl!(<K, V, H> JsonSchema for std::collections::HashMap<K, V, H>);
