use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use std::time::{Duration, SystemTime};

impl JsonSchema for Duration {
    fn schema_name() -> String {
        "Duration".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            ..Default::default()
        };
        let obj = schema.object();
        obj.required.insert("secs".to_owned());
        obj.required.insert("nanos".to_owned());
        obj.properties
            .insert("secs".to_owned(), <u64>::json_schema(gen));
        obj.properties
            .insert("nanos".to_owned(), <u32>::json_schema(gen));
        schema.into()
    }
}

impl JsonSchema for SystemTime {
    fn schema_name() -> String {
        "SystemTime".to_owned()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            ..Default::default()
        };
        let obj = schema.object();
        obj.required.insert("secs_since_epoch".to_owned());
        obj.required.insert("nanos_since_epoch".to_owned());
        obj.properties
            .insert("secs_since_epoch".to_owned(), <u64>::json_schema(gen));
        obj.properties
            .insert("nanos_since_epoch".to_owned(), <u32>::json_schema(gen));
        schema.into()
    }
}
