use crate::gen::SchemaGenerator;
use crate::schema::*;
use crate::JsonSchema;
use either::Either;

impl<L: JsonSchema, R: JsonSchema> JsonSchema for Either<L, R> {
    no_ref_schema!();

    fn schema_name() -> String {
        format!("Either_{}_or_{}", L::schema_name(), R::schema_name())
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema = SchemaObject::default();
        schema.subschemas().any_of = Some(vec![gen.subschema_for::<L>(), gen.subschema_for::<R>()]);
        schema.into()
    }
}
