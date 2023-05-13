// @generated
impl serde::Serialize for Module {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.bech32_prefix.is_empty() {
            len += 1;
        }
        if !self.module_account_permissions.is_empty() {
            len += 1;
        }
        if !self.authority.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.auth.module.v1.Module", len)?;
        if !self.bech32_prefix.is_empty() {
            struct_ser.serialize_field("bech32Prefix", &self.bech32_prefix)?;
        }
        if !self.module_account_permissions.is_empty() {
            struct_ser.serialize_field("moduleAccountPermissions", &self.module_account_permissions)?;
        }
        if !self.authority.is_empty() {
            struct_ser.serialize_field("authority", &self.authority)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Module {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "bech32_prefix",
            "bech32Prefix",
            "module_account_permissions",
            "moduleAccountPermissions",
            "authority",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Bech32Prefix,
            ModuleAccountPermissions,
            Authority,
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
                            "bech32Prefix" | "bech32_prefix" => Ok(GeneratedField::Bech32Prefix),
                            "moduleAccountPermissions" | "module_account_permissions" => Ok(GeneratedField::ModuleAccountPermissions),
                            "authority" => Ok(GeneratedField::Authority),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Module;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.auth.module.v1.Module")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Module, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut bech32_prefix__ = None;
                let mut module_account_permissions__ = None;
                let mut authority__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Bech32Prefix => {
                            if bech32_prefix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bech32Prefix"));
                            }
                            bech32_prefix__ = Some(map.next_value()?);
                        }
                        GeneratedField::ModuleAccountPermissions => {
                            if module_account_permissions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("moduleAccountPermissions"));
                            }
                            module_account_permissions__ = Some(map.next_value()?);
                        }
                        GeneratedField::Authority => {
                            if authority__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authority"));
                            }
                            authority__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Module {
                    bech32_prefix: bech32_prefix__.unwrap_or_default(),
                    module_account_permissions: module_account_permissions__.unwrap_or_default(),
                    authority: authority__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.auth.module.v1.Module", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ModuleAccountPermission {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.account.is_empty() {
            len += 1;
        }
        if !self.permissions.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.auth.module.v1.ModuleAccountPermission", len)?;
        if !self.account.is_empty() {
            struct_ser.serialize_field("account", &self.account)?;
        }
        if !self.permissions.is_empty() {
            struct_ser.serialize_field("permissions", &self.permissions)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ModuleAccountPermission {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "account",
            "permissions",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Account,
            Permissions,
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
                            "account" => Ok(GeneratedField::Account),
                            "permissions" => Ok(GeneratedField::Permissions),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ModuleAccountPermission;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.auth.module.v1.ModuleAccountPermission")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ModuleAccountPermission, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut account__ = None;
                let mut permissions__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Account => {
                            if account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("account"));
                            }
                            account__ = Some(map.next_value()?);
                        }
                        GeneratedField::Permissions => {
                            if permissions__.is_some() {
                                return Err(serde::de::Error::duplicate_field("permissions"));
                            }
                            permissions__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ModuleAccountPermission {
                    account: account__.unwrap_or_default(),
                    permissions: permissions__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.auth.module.v1.ModuleAccountPermission", FIELDS, GeneratedVisitor)
    }
}
