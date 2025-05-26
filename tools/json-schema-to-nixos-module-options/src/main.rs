use std::{env, fs, mem};

use schemars::{
    schema::{InstanceType, Metadata, RootSchema, Schema, SchemaObject, SingleOrVec},
    visit::Visitor,
};
use serde_json::{Map, Number, Value};

fn main() {
    let mut root_schema = serde_json::from_str::<RootSchema>(
        &fs::read_to_string(env::args().nth(1).unwrap()).unwrap(),
    )
    .unwrap();

    let mut visitor = JsonSchemaToNixosModuleOptions {
        output: String::new(),
        writing_root_object: true,
    };

    visitor.visit_root_schema(&mut root_schema);

    println!("{}", visitor.output);
}

struct JsonSchemaToNixosModuleOptions {
    output: String,
    writing_root_object: bool,
}

impl Visitor for JsonSchemaToNixosModuleOptions {
    fn visit_root_schema(&mut self, root: &mut RootSchema) {
        self.output += "{ types, mkOption }: let\ndefinitions = {";

        self.writing_root_object = false;

        for (name, schema) in &mut root.definitions {
            self.output += &format!(r##""#/definitions/{name}" = "##);
            self.visit_schema(schema);
            self.output += ";\n";
        }

        self.output += "};\nin\n";

        self.writing_root_object = true;

        self.visit_schema_object(&mut root.schema);
    }

    fn visit_schema(&mut self, schema: &mut Schema) {
        match schema {
            Schema::Bool(true) => {
                self.visit_schema_object(&mut SchemaObject::default());
            }
            Schema::Bool(false) => {
                todo!("what does this even mean");
            }
            Schema::Object(obj) => self.visit_schema_object(obj),
        }
    }

    fn visit_schema_object(&mut self, schema: &mut SchemaObject) {
        let writing_root = mem::replace(&mut self.writing_root_object, false);

        match (&mut schema.instance_type, &mut schema.reference) {
            (Some(instance_type), None) => match instance_type {
                SingleOrVec::Single(ty) => match &**ty {
                    InstanceType::Null => todo!(),
                    InstanceType::Boolean => {
                        self.output += "types.bool";
                    }
                    InstanceType::Object => {
                        let obj_val = schema.object.as_mut().unwrap();
                        if writing_root {
                            self.output += "{";
                        } else {
                            self.output += "types.submodule { options = {";
                        };
                        for (property_name, property) in &mut obj_val.properties {
                            self.output += &format!(r#""{property_name}" = mkOption {{ type = "#);
                            let optional_property = !obj_val.required.contains(property_name);

                            let metadata = match property {
                                Schema::Bool(_) => Metadata::default(),
                                Schema::Object(schema_object) => {
                                    *schema_object.metadata.clone().unwrap_or_default()
                                }
                            };

                            let property_obj = property.clone().into_object();

                            let nullable = property_obj
                                .extensions
                                .get("nullable")
                                .is_some_and(|e| e.as_bool().unwrap_or_default());

                            if optional_property && nullable && metadata.default.is_none() {
                                self.output += "types.nullOr (";
                            }

                            self.visit_schema(property);

                            if optional_property && nullable && metadata.default.is_none() {
                                self.output += ")";
                            }

                            self.output += ";";

                            if let Some(default_value) = metadata.default {
                                self.output += "default = ";
                                self.output += &json_value_to_nix_value(default_value);
                                self.output += ";";
                            } else if optional_property && !nullable {
                                self.output += "default = ";
                                self.output += &json_value_to_nix_value(
                                    match property_obj.instance_type.unwrap() {
                                        SingleOrVec::Single(ty) => match *ty {
                                            InstanceType::Null => Value::Null,
                                            InstanceType::Boolean => Value::Bool(false),
                                            InstanceType::Object => Value::Object(Map::new()),
                                            InstanceType::Array => Value::Array(vec![]),
                                            InstanceType::Number => {
                                                Value::Number(Number::from_f64(0.0).unwrap())
                                            }
                                            InstanceType::String => Value::String("".to_owned()),
                                            InstanceType::Integer => {
                                                Value::Number(Number::from_u128(0).unwrap())
                                            }
                                        },
                                        SingleOrVec::Vec(_) => todo!(),
                                    },
                                );
                                self.output += ";";
                            }

                            self.output += "};\n";
                        }
                        if writing_root {
                            self.output += "}";
                        } else {
                            self.output += "};}";
                        };
                    }
                    InstanceType::Array => {
                        self.output += "types.listOf (";

                        match schema.array.as_mut().unwrap().items.as_mut().unwrap() {
                            SingleOrVec::Single(item_schema) => self.visit_schema(item_schema),
                            SingleOrVec::Vec(_) => todo!(),
                        };

                        self.output += ")";
                    }
                    InstanceType::Number => {
                        self.output += "types.number";
                    }
                    InstanceType::String => {
                        self.output += "types.str";
                    }
                    InstanceType::Integer => {
                        self.output += "types.int";
                    }
                },
                SingleOrVec::Vec(_) => todo!(),
            },
            (None, Some(reference)) => {
                self.output += "definitions.\"";
                self.output += reference;
                self.output += "\"";
            }
            (None, None) => {
                self.output += "types.attrs";
            }
            _ => {
                println!("{}", self.output);

                todo!("{schema:#?}");
            }
        }

        self.writing_root_object = writing_root;
    }
}

fn json_value_to_nix_value(value: Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => format!(r#""{s}""#),
        Value::Array(a) => format!(
            "[{}]",
            a.into_iter()
                .map(json_value_to_nix_value)
                .map(|v| format!("({v})"))
                .collect::<String>()
        ),
        Value::Object(o) => format!(
            "{{{}}}",
            o.into_iter()
                .map(|(k, v)| format!(r#""{k}" = {};"#, json_value_to_nix_value(v)))
                .collect::<String>()
        ),
    }
}
