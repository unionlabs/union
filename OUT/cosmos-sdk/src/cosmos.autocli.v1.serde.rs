// @generated
impl serde::Serialize for AppOptionsRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.autocli.v1.AppOptionsRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppOptionsRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                            Err(serde::de::Error::unknown_field(value, FIELDS))
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppOptionsRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.autocli.v1.AppOptionsRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AppOptionsRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(AppOptionsRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.autocli.v1.AppOptionsRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AppOptionsResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.module_options.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.autocli.v1.AppOptionsResponse", len)?;
        if !self.module_options.is_empty() {
            struct_ser.serialize_field("moduleOptions", &self.module_options)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppOptionsResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "module_options",
            "moduleOptions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ModuleOptions,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "moduleOptions" | "module_options" => Ok(GeneratedField::ModuleOptions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppOptionsResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.autocli.v1.AppOptionsResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AppOptionsResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut module_options__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ModuleOptions => {
                            if module_options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("moduleOptions"));
                            }
                            module_options__ = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(AppOptionsResponse {
                    module_options: module_options__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.autocli.v1.AppOptionsResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for FlagOptions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.name.is_empty() {
            len += 1;
        }
        if !self.shorthand.is_empty() {
            len += 1;
        }
        if !self.usage.is_empty() {
            len += 1;
        }
        if !self.default_value.is_empty() {
            len += 1;
        }
        if !self.no_opt_default_value.is_empty() {
            len += 1;
        }
        if !self.deprecated.is_empty() {
            len += 1;
        }
        if !self.shorthand_deprecated.is_empty() {
            len += 1;
        }
        if self.hidden {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.autocli.v1.FlagOptions", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.shorthand.is_empty() {
            struct_ser.serialize_field("shorthand", &self.shorthand)?;
        }
        if !self.usage.is_empty() {
            struct_ser.serialize_field("usage", &self.usage)?;
        }
        if !self.default_value.is_empty() {
            struct_ser.serialize_field("defaultValue", &self.default_value)?;
        }
        if !self.no_opt_default_value.is_empty() {
            struct_ser.serialize_field("noOptDefaultValue", &self.no_opt_default_value)?;
        }
        if !self.deprecated.is_empty() {
            struct_ser.serialize_field("deprecated", &self.deprecated)?;
        }
        if !self.shorthand_deprecated.is_empty() {
            struct_ser.serialize_field("shorthandDeprecated", &self.shorthand_deprecated)?;
        }
        if self.hidden {
            struct_ser.serialize_field("hidden", &self.hidden)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for FlagOptions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "shorthand",
            "usage",
            "default_value",
            "defaultValue",
            "no_opt_default_value",
            "noOptDefaultValue",
            "deprecated",
            "shorthand_deprecated",
            "shorthandDeprecated",
            "hidden",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Shorthand,
            Usage,
            DefaultValue,
            NoOptDefaultValue,
            Deprecated,
            ShorthandDeprecated,
            Hidden,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "name" => Ok(GeneratedField::Name),
                            "shorthand" => Ok(GeneratedField::Shorthand),
                            "usage" => Ok(GeneratedField::Usage),
                            "defaultValue" | "default_value" => Ok(GeneratedField::DefaultValue),
                            "noOptDefaultValue" | "no_opt_default_value" => Ok(GeneratedField::NoOptDefaultValue),
                            "deprecated" => Ok(GeneratedField::Deprecated),
                            "shorthandDeprecated" | "shorthand_deprecated" => Ok(GeneratedField::ShorthandDeprecated),
                            "hidden" => Ok(GeneratedField::Hidden),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = FlagOptions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.autocli.v1.FlagOptions")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<FlagOptions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut shorthand__ = None;
                let mut usage__ = None;
                let mut default_value__ = None;
                let mut no_opt_default_value__ = None;
                let mut deprecated__ = None;
                let mut shorthand_deprecated__ = None;
                let mut hidden__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Shorthand => {
                            if shorthand__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shorthand"));
                            }
                            shorthand__ = Some(map.next_value()?);
                        }
                        GeneratedField::Usage => {
                            if usage__.is_some() {
                                return Err(serde::de::Error::duplicate_field("usage"));
                            }
                            usage__ = Some(map.next_value()?);
                        }
                        GeneratedField::DefaultValue => {
                            if default_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("defaultValue"));
                            }
                            default_value__ = Some(map.next_value()?);
                        }
                        GeneratedField::NoOptDefaultValue => {
                            if no_opt_default_value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noOptDefaultValue"));
                            }
                            no_opt_default_value__ = Some(map.next_value()?);
                        }
                        GeneratedField::Deprecated => {
                            if deprecated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deprecated"));
                            }
                            deprecated__ = Some(map.next_value()?);
                        }
                        GeneratedField::ShorthandDeprecated => {
                            if shorthand_deprecated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("shorthandDeprecated"));
                            }
                            shorthand_deprecated__ = Some(map.next_value()?);
                        }
                        GeneratedField::Hidden => {
                            if hidden__.is_some() {
                                return Err(serde::de::Error::duplicate_field("hidden"));
                            }
                            hidden__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(FlagOptions {
                    name: name__.unwrap_or_default(),
                    shorthand: shorthand__.unwrap_or_default(),
                    usage: usage__.unwrap_or_default(),
                    default_value: default_value__.unwrap_or_default(),
                    no_opt_default_value: no_opt_default_value__.unwrap_or_default(),
                    deprecated: deprecated__.unwrap_or_default(),
                    shorthand_deprecated: shorthand_deprecated__.unwrap_or_default(),
                    hidden: hidden__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.autocli.v1.FlagOptions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModuleOptions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tx.is_some() {
            len += 1;
        }
        if self.query.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.autocli.v1.ModuleOptions", len)?;
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        if let Some(v) = self.query.as_ref() {
            struct_ser.serialize_field("query", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModuleOptions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx",
            "query",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tx,
            Query,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "tx" => Ok(GeneratedField::Tx),
                            "query" => Ok(GeneratedField::Query),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModuleOptions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.autocli.v1.ModuleOptions")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ModuleOptions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx__ = None;
                let mut query__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map.next_value()?;
                        }
                        GeneratedField::Query => {
                            if query__.is_some() {
                                return Err(serde::de::Error::duplicate_field("query"));
                            }
                            query__ = map.next_value()?;
                        }
                    }
                }
                Ok(ModuleOptions {
                    tx: tx__,
                    query: query__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.autocli.v1.ModuleOptions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PositionalArgDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proto_field.is_empty() {
            len += 1;
        }
        if self.varargs {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.autocli.v1.PositionalArgDescriptor", len)?;
        if !self.proto_field.is_empty() {
            struct_ser.serialize_field("protoField", &self.proto_field)?;
        }
        if self.varargs {
            struct_ser.serialize_field("varargs", &self.varargs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PositionalArgDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proto_field",
            "protoField",
            "varargs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProtoField,
            Varargs,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "protoField" | "proto_field" => Ok(GeneratedField::ProtoField),
                            "varargs" => Ok(GeneratedField::Varargs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PositionalArgDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.autocli.v1.PositionalArgDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PositionalArgDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proto_field__ = None;
                let mut varargs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProtoField => {
                            if proto_field__.is_some() {
                                return Err(serde::de::Error::duplicate_field("protoField"));
                            }
                            proto_field__ = Some(map.next_value()?);
                        }
                        GeneratedField::Varargs => {
                            if varargs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("varargs"));
                            }
                            varargs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PositionalArgDescriptor {
                    proto_field: proto_field__.unwrap_or_default(),
                    varargs: varargs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.autocli.v1.PositionalArgDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RpcCommandOptions {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.rpc_method.is_empty() {
            len += 1;
        }
        if !self.r#use.is_empty() {
            len += 1;
        }
        if !self.long.is_empty() {
            len += 1;
        }
        if !self.short.is_empty() {
            len += 1;
        }
        if !self.example.is_empty() {
            len += 1;
        }
        if !self.alias.is_empty() {
            len += 1;
        }
        if !self.suggest_for.is_empty() {
            len += 1;
        }
        if !self.deprecated.is_empty() {
            len += 1;
        }
        if !self.version.is_empty() {
            len += 1;
        }
        if !self.flag_options.is_empty() {
            len += 1;
        }
        if !self.positional_args.is_empty() {
            len += 1;
        }
        if self.skip {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.autocli.v1.RpcCommandOptions", len)?;
        if !self.rpc_method.is_empty() {
            struct_ser.serialize_field("rpcMethod", &self.rpc_method)?;
        }
        if !self.r#use.is_empty() {
            struct_ser.serialize_field("use", &self.r#use)?;
        }
        if !self.long.is_empty() {
            struct_ser.serialize_field("long", &self.long)?;
        }
        if !self.short.is_empty() {
            struct_ser.serialize_field("short", &self.short)?;
        }
        if !self.example.is_empty() {
            struct_ser.serialize_field("example", &self.example)?;
        }
        if !self.alias.is_empty() {
            struct_ser.serialize_field("alias", &self.alias)?;
        }
        if !self.suggest_for.is_empty() {
            struct_ser.serialize_field("suggestFor", &self.suggest_for)?;
        }
        if !self.deprecated.is_empty() {
            struct_ser.serialize_field("deprecated", &self.deprecated)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
        }
        if !self.flag_options.is_empty() {
            struct_ser.serialize_field("flagOptions", &self.flag_options)?;
        }
        if !self.positional_args.is_empty() {
            struct_ser.serialize_field("positionalArgs", &self.positional_args)?;
        }
        if self.skip {
            struct_ser.serialize_field("skip", &self.skip)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RpcCommandOptions {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "rpc_method",
            "rpcMethod",
            "use",
            "long",
            "short",
            "example",
            "alias",
            "suggest_for",
            "suggestFor",
            "deprecated",
            "version",
            "flag_options",
            "flagOptions",
            "positional_args",
            "positionalArgs",
            "skip",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            RpcMethod,
            Use,
            Long,
            Short,
            Example,
            Alias,
            SuggestFor,
            Deprecated,
            Version,
            FlagOptions,
            PositionalArgs,
            Skip,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "rpcMethod" | "rpc_method" => Ok(GeneratedField::RpcMethod),
                            "use" => Ok(GeneratedField::Use),
                            "long" => Ok(GeneratedField::Long),
                            "short" => Ok(GeneratedField::Short),
                            "example" => Ok(GeneratedField::Example),
                            "alias" => Ok(GeneratedField::Alias),
                            "suggestFor" | "suggest_for" => Ok(GeneratedField::SuggestFor),
                            "deprecated" => Ok(GeneratedField::Deprecated),
                            "version" => Ok(GeneratedField::Version),
                            "flagOptions" | "flag_options" => Ok(GeneratedField::FlagOptions),
                            "positionalArgs" | "positional_args" => Ok(GeneratedField::PositionalArgs),
                            "skip" => Ok(GeneratedField::Skip),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RpcCommandOptions;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.autocli.v1.RpcCommandOptions")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RpcCommandOptions, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut rpc_method__ = None;
                let mut r#use__ = None;
                let mut long__ = None;
                let mut short__ = None;
                let mut example__ = None;
                let mut alias__ = None;
                let mut suggest_for__ = None;
                let mut deprecated__ = None;
                let mut version__ = None;
                let mut flag_options__ = None;
                let mut positional_args__ = None;
                let mut skip__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::RpcMethod => {
                            if rpc_method__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rpcMethod"));
                            }
                            rpc_method__ = Some(map.next_value()?);
                        }
                        GeneratedField::Use => {
                            if r#use__.is_some() {
                                return Err(serde::de::Error::duplicate_field("use"));
                            }
                            r#use__ = Some(map.next_value()?);
                        }
                        GeneratedField::Long => {
                            if long__.is_some() {
                                return Err(serde::de::Error::duplicate_field("long"));
                            }
                            long__ = Some(map.next_value()?);
                        }
                        GeneratedField::Short => {
                            if short__.is_some() {
                                return Err(serde::de::Error::duplicate_field("short"));
                            }
                            short__ = Some(map.next_value()?);
                        }
                        GeneratedField::Example => {
                            if example__.is_some() {
                                return Err(serde::de::Error::duplicate_field("example"));
                            }
                            example__ = Some(map.next_value()?);
                        }
                        GeneratedField::Alias => {
                            if alias__.is_some() {
                                return Err(serde::de::Error::duplicate_field("alias"));
                            }
                            alias__ = Some(map.next_value()?);
                        }
                        GeneratedField::SuggestFor => {
                            if suggest_for__.is_some() {
                                return Err(serde::de::Error::duplicate_field("suggestFor"));
                            }
                            suggest_for__ = Some(map.next_value()?);
                        }
                        GeneratedField::Deprecated => {
                            if deprecated__.is_some() {
                                return Err(serde::de::Error::duplicate_field("deprecated"));
                            }
                            deprecated__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map.next_value()?);
                        }
                        GeneratedField::FlagOptions => {
                            if flag_options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("flagOptions"));
                            }
                            flag_options__ = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                        GeneratedField::PositionalArgs => {
                            if positional_args__.is_some() {
                                return Err(serde::de::Error::duplicate_field("positionalArgs"));
                            }
                            positional_args__ = Some(map.next_value()?);
                        }
                        GeneratedField::Skip => {
                            if skip__.is_some() {
                                return Err(serde::de::Error::duplicate_field("skip"));
                            }
                            skip__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RpcCommandOptions {
                    rpc_method: rpc_method__.unwrap_or_default(),
                    r#use: r#use__.unwrap_or_default(),
                    long: long__.unwrap_or_default(),
                    short: short__.unwrap_or_default(),
                    example: example__.unwrap_or_default(),
                    alias: alias__.unwrap_or_default(),
                    suggest_for: suggest_for__.unwrap_or_default(),
                    deprecated: deprecated__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    flag_options: flag_options__.unwrap_or_default(),
                    positional_args: positional_args__.unwrap_or_default(),
                    skip: skip__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.autocli.v1.RpcCommandOptions", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ServiceCommandDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.service.is_empty() {
            len += 1;
        }
        if !self.rpc_command_options.is_empty() {
            len += 1;
        }
        if !self.sub_commands.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.autocli.v1.ServiceCommandDescriptor", len)?;
        if !self.service.is_empty() {
            struct_ser.serialize_field("service", &self.service)?;
        }
        if !self.rpc_command_options.is_empty() {
            struct_ser.serialize_field("rpcCommandOptions", &self.rpc_command_options)?;
        }
        if !self.sub_commands.is_empty() {
            struct_ser.serialize_field("subCommands", &self.sub_commands)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ServiceCommandDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "service",
            "rpc_command_options",
            "rpcCommandOptions",
            "sub_commands",
            "subCommands",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Service,
            RpcCommandOptions,
            SubCommands,
        }
        impl<'de> serde::Deserialize<'de> for GeneratedField {
            fn deserialize<D>(deserializer: D) -> std::result::Result<GeneratedField, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct GeneratedVisitor;

                impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
                    type Value = GeneratedField;

                    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        write!(formatter, "expected one of: {:?}", &FIELDS)
                    }

                    #[allow(unused_variables)]
                    fn visit_str<E>(self, value: &str) -> std::result::Result<GeneratedField, E>
                    where
                        E: serde::de::Error,
                    {
                        match value {
                            "service" => Ok(GeneratedField::Service),
                            "rpcCommandOptions" | "rpc_command_options" => Ok(GeneratedField::RpcCommandOptions),
                            "subCommands" | "sub_commands" => Ok(GeneratedField::SubCommands),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ServiceCommandDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.autocli.v1.ServiceCommandDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ServiceCommandDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut service__ = None;
                let mut rpc_command_options__ = None;
                let mut sub_commands__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Service => {
                            if service__.is_some() {
                                return Err(serde::de::Error::duplicate_field("service"));
                            }
                            service__ = Some(map.next_value()?);
                        }
                        GeneratedField::RpcCommandOptions => {
                            if rpc_command_options__.is_some() {
                                return Err(serde::de::Error::duplicate_field("rpcCommandOptions"));
                            }
                            rpc_command_options__ = Some(map.next_value()?);
                        }
                        GeneratedField::SubCommands => {
                            if sub_commands__.is_some() {
                                return Err(serde::de::Error::duplicate_field("subCommands"));
                            }
                            sub_commands__ = Some(
                                map.next_value::<std::collections::HashMap<_, _>>()?
                            );
                        }
                    }
                }
                Ok(ServiceCommandDescriptor {
                    service: service__.unwrap_or_default(),
                    rpc_command_options: rpc_command_options__.unwrap_or_default(),
                    sub_commands: sub_commands__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.autocli.v1.ServiceCommandDescriptor", FIELDS, GeneratedVisitor)
    }
}
