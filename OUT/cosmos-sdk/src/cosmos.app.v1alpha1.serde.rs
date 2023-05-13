// @generated
impl serde::Serialize for Config {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.modules.is_empty() {
            len += 1;
        }
        if !self.golang_bindings.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.Config", len)?;
        if !self.modules.is_empty() {
            struct_ser.serialize_field("modules", &self.modules)?;
        }
        if !self.golang_bindings.is_empty() {
            struct_ser.serialize_field("golangBindings", &self.golang_bindings)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Config {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "modules",
            "golang_bindings",
            "golangBindings",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Modules,
            GolangBindings,
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
                            "modules" => Ok(GeneratedField::Modules),
                            "golangBindings" | "golang_bindings" => Ok(GeneratedField::GolangBindings),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Config;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.Config")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Config, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut modules__ = None;
                let mut golang_bindings__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Modules => {
                            if modules__.is_some() {
                                return Err(serde::de::Error::duplicate_field("modules"));
                            }
                            modules__ = Some(map.next_value()?);
                        }
                        GeneratedField::GolangBindings => {
                            if golang_bindings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("golangBindings"));
                            }
                            golang_bindings__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Config {
                    modules: modules__.unwrap_or_default(),
                    golang_bindings: golang_bindings__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.Config", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GolangBinding {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.interface_type.is_empty() {
            len += 1;
        }
        if !self.implementation.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.GolangBinding", len)?;
        if !self.interface_type.is_empty() {
            struct_ser.serialize_field("interfaceType", &self.interface_type)?;
        }
        if !self.implementation.is_empty() {
            struct_ser.serialize_field("implementation", &self.implementation)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GolangBinding {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "interface_type",
            "interfaceType",
            "implementation",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            InterfaceType,
            Implementation,
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
                            "interfaceType" | "interface_type" => Ok(GeneratedField::InterfaceType),
                            "implementation" => Ok(GeneratedField::Implementation),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GolangBinding;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.GolangBinding")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GolangBinding, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut interface_type__ = None;
                let mut implementation__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::InterfaceType => {
                            if interface_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interfaceType"));
                            }
                            interface_type__ = Some(map.next_value()?);
                        }
                        GeneratedField::Implementation => {
                            if implementation__.is_some() {
                                return Err(serde::de::Error::duplicate_field("implementation"));
                            }
                            implementation__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GolangBinding {
                    interface_type: interface_type__.unwrap_or_default(),
                    implementation: implementation__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.GolangBinding", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MigrateFromInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.module.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.MigrateFromInfo", len)?;
        if !self.module.is_empty() {
            struct_ser.serialize_field("module", &self.module)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MigrateFromInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "module",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Module,
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
                            "module" => Ok(GeneratedField::Module),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MigrateFromInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.MigrateFromInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MigrateFromInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut module__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Module => {
                            if module__.is_some() {
                                return Err(serde::de::Error::duplicate_field("module"));
                            }
                            module__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MigrateFromInfo {
                    module: module__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.MigrateFromInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModuleConfig {
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
        if self.config.is_some() {
            len += 1;
        }
        if !self.golang_bindings.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.ModuleConfig", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if let Some(v) = self.config.as_ref() {
            struct_ser.serialize_field("config", v)?;
        }
        if !self.golang_bindings.is_empty() {
            struct_ser.serialize_field("golangBindings", &self.golang_bindings)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModuleConfig {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "config",
            "golang_bindings",
            "golangBindings",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Config,
            GolangBindings,
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
                            "config" => Ok(GeneratedField::Config),
                            "golangBindings" | "golang_bindings" => Ok(GeneratedField::GolangBindings),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModuleConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.ModuleConfig")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ModuleConfig, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut config__ = None;
                let mut golang_bindings__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Config => {
                            if config__.is_some() {
                                return Err(serde::de::Error::duplicate_field("config"));
                            }
                            config__ = map.next_value()?;
                        }
                        GeneratedField::GolangBindings => {
                            if golang_bindings__.is_some() {
                                return Err(serde::de::Error::duplicate_field("golangBindings"));
                            }
                            golang_bindings__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ModuleConfig {
                    name: name__.unwrap_or_default(),
                    config: config__,
                    golang_bindings: golang_bindings__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.ModuleConfig", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModuleDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.go_import.is_empty() {
            len += 1;
        }
        if !self.use_package.is_empty() {
            len += 1;
        }
        if !self.can_migrate_from.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.ModuleDescriptor", len)?;
        if !self.go_import.is_empty() {
            struct_ser.serialize_field("goImport", &self.go_import)?;
        }
        if !self.use_package.is_empty() {
            struct_ser.serialize_field("usePackage", &self.use_package)?;
        }
        if !self.can_migrate_from.is_empty() {
            struct_ser.serialize_field("canMigrateFrom", &self.can_migrate_from)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModuleDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "go_import",
            "goImport",
            "use_package",
            "usePackage",
            "can_migrate_from",
            "canMigrateFrom",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GoImport,
            UsePackage,
            CanMigrateFrom,
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
                            "goImport" | "go_import" => Ok(GeneratedField::GoImport),
                            "usePackage" | "use_package" => Ok(GeneratedField::UsePackage),
                            "canMigrateFrom" | "can_migrate_from" => Ok(GeneratedField::CanMigrateFrom),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModuleDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.ModuleDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ModuleDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut go_import__ = None;
                let mut use_package__ = None;
                let mut can_migrate_from__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GoImport => {
                            if go_import__.is_some() {
                                return Err(serde::de::Error::duplicate_field("goImport"));
                            }
                            go_import__ = Some(map.next_value()?);
                        }
                        GeneratedField::UsePackage => {
                            if use_package__.is_some() {
                                return Err(serde::de::Error::duplicate_field("usePackage"));
                            }
                            use_package__ = Some(map.next_value()?);
                        }
                        GeneratedField::CanMigrateFrom => {
                            if can_migrate_from__.is_some() {
                                return Err(serde::de::Error::duplicate_field("canMigrateFrom"));
                            }
                            can_migrate_from__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ModuleDescriptor {
                    go_import: go_import__.unwrap_or_default(),
                    use_package: use_package__.unwrap_or_default(),
                    can_migrate_from: can_migrate_from__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.ModuleDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PackageReference {
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
        if self.revision != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.PackageReference", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.revision != 0 {
            struct_ser.serialize_field("revision", &self.revision)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PackageReference {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "revision",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Revision,
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
                            "revision" => Ok(GeneratedField::Revision),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PackageReference;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.PackageReference")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PackageReference, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut revision__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Revision => {
                            if revision__.is_some() {
                                return Err(serde::de::Error::duplicate_field("revision"));
                            }
                            revision__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(PackageReference {
                    name: name__.unwrap_or_default(),
                    revision: revision__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.PackageReference", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConfigRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.QueryConfigRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConfigRequest {
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
            type Value = QueryConfigRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.QueryConfigRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConfigRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(QueryConfigRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.QueryConfigRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryConfigResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.config.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.app.v1alpha1.QueryConfigResponse", len)?;
        if let Some(v) = self.config.as_ref() {
            struct_ser.serialize_field("config", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryConfigResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "config",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Config,
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
                            "config" => Ok(GeneratedField::Config),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryConfigResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.app.v1alpha1.QueryConfigResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryConfigResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut config__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Config => {
                            if config__.is_some() {
                                return Err(serde::de::Error::duplicate_field("config"));
                            }
                            config__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryConfigResponse {
                    config: config__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.app.v1alpha1.QueryConfigResponse", FIELDS, GeneratedVisitor)
    }
}
