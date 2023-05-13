// @generated
impl serde::Serialize for AppDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.authn.is_some() {
            len += 1;
        }
        if self.chain.is_some() {
            len += 1;
        }
        if self.codec.is_some() {
            len += 1;
        }
        if self.configuration.is_some() {
            len += 1;
        }
        if self.query_services.is_some() {
            len += 1;
        }
        if self.tx.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.AppDescriptor", len)?;
        if let Some(v) = self.authn.as_ref() {
            struct_ser.serialize_field("authn", v)?;
        }
        if let Some(v) = self.chain.as_ref() {
            struct_ser.serialize_field("chain", v)?;
        }
        if let Some(v) = self.codec.as_ref() {
            struct_ser.serialize_field("codec", v)?;
        }
        if let Some(v) = self.configuration.as_ref() {
            struct_ser.serialize_field("configuration", v)?;
        }
        if let Some(v) = self.query_services.as_ref() {
            struct_ser.serialize_field("queryServices", v)?;
        }
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AppDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authn",
            "chain",
            "codec",
            "configuration",
            "query_services",
            "queryServices",
            "tx",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Authn,
            Chain,
            Codec,
            Configuration,
            QueryServices,
            Tx,
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
                            "authn" => Ok(GeneratedField::Authn),
                            "chain" => Ok(GeneratedField::Chain),
                            "codec" => Ok(GeneratedField::Codec),
                            "configuration" => Ok(GeneratedField::Configuration),
                            "queryServices" | "query_services" => Ok(GeneratedField::QueryServices),
                            "tx" => Ok(GeneratedField::Tx),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AppDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.AppDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AppDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authn__ = None;
                let mut chain__ = None;
                let mut codec__ = None;
                let mut configuration__ = None;
                let mut query_services__ = None;
                let mut tx__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Authn => {
                            if authn__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authn"));
                            }
                            authn__ = map.next_value()?;
                        }
                        GeneratedField::Chain => {
                            if chain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chain"));
                            }
                            chain__ = map.next_value()?;
                        }
                        GeneratedField::Codec => {
                            if codec__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codec"));
                            }
                            codec__ = map.next_value()?;
                        }
                        GeneratedField::Configuration => {
                            if configuration__.is_some() {
                                return Err(serde::de::Error::duplicate_field("configuration"));
                            }
                            configuration__ = map.next_value()?;
                        }
                        GeneratedField::QueryServices => {
                            if query_services__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryServices"));
                            }
                            query_services__ = map.next_value()?;
                        }
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map.next_value()?;
                        }
                    }
                }
                Ok(AppDescriptor {
                    authn: authn__,
                    chain: chain__,
                    codec: codec__,
                    configuration: configuration__,
                    query_services: query_services__,
                    tx: tx__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.AppDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for AuthnDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.sign_modes.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.AuthnDescriptor", len)?;
        if !self.sign_modes.is_empty() {
            struct_ser.serialize_field("signModes", &self.sign_modes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AuthnDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "sign_modes",
            "signModes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SignModes,
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
                            "signModes" | "sign_modes" => Ok(GeneratedField::SignModes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AuthnDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.AuthnDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<AuthnDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sign_modes__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::SignModes => {
                            if sign_modes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signModes"));
                            }
                            sign_modes__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(AuthnDescriptor {
                    sign_modes: sign_modes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.AuthnDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ChainDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.ChainDescriptor", len)?;
        if !self.id.is_empty() {
            struct_ser.serialize_field("id", &self.id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ChainDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
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
                            "id" => Ok(GeneratedField::Id),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ChainDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.ChainDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ChainDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ChainDescriptor {
                    id: id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.ChainDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for CodecDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.interfaces.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.CodecDescriptor", len)?;
        if !self.interfaces.is_empty() {
            struct_ser.serialize_field("interfaces", &self.interfaces)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for CodecDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "interfaces",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Interfaces,
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
                            "interfaces" => Ok(GeneratedField::Interfaces),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = CodecDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.CodecDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<CodecDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut interfaces__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Interfaces => {
                            if interfaces__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interfaces"));
                            }
                            interfaces__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(CodecDescriptor {
                    interfaces: interfaces__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.CodecDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ConfigurationDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.bech32_account_address_prefix.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.ConfigurationDescriptor", len)?;
        if !self.bech32_account_address_prefix.is_empty() {
            struct_ser.serialize_field("bech32AccountAddressPrefix", &self.bech32_account_address_prefix)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ConfigurationDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "bech32_account_address_prefix",
            "bech32AccountAddressPrefix",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Bech32AccountAddressPrefix,
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
                            "bech32AccountAddressPrefix" | "bech32_account_address_prefix" => Ok(GeneratedField::Bech32AccountAddressPrefix),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ConfigurationDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.ConfigurationDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ConfigurationDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut bech32_account_address_prefix__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Bech32AccountAddressPrefix => {
                            if bech32_account_address_prefix__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bech32AccountAddressPrefix"));
                            }
                            bech32_account_address_prefix__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(ConfigurationDescriptor {
                    bech32_account_address_prefix: bech32_account_address_prefix__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.ConfigurationDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetAuthnDescriptorRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetAuthnDescriptorRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetAuthnDescriptorRequest {
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
            type Value = GetAuthnDescriptorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetAuthnDescriptorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetAuthnDescriptorRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetAuthnDescriptorRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetAuthnDescriptorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetAuthnDescriptorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.authn.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetAuthnDescriptorResponse", len)?;
        if let Some(v) = self.authn.as_ref() {
            struct_ser.serialize_field("authn", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetAuthnDescriptorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "authn",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Authn,
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
                            "authn" => Ok(GeneratedField::Authn),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetAuthnDescriptorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetAuthnDescriptorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetAuthnDescriptorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut authn__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Authn => {
                            if authn__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authn"));
                            }
                            authn__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetAuthnDescriptorResponse {
                    authn: authn__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetAuthnDescriptorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetChainDescriptorRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetChainDescriptorRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetChainDescriptorRequest {
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
            type Value = GetChainDescriptorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetChainDescriptorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetChainDescriptorRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetChainDescriptorRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetChainDescriptorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetChainDescriptorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.chain.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetChainDescriptorResponse", len)?;
        if let Some(v) = self.chain.as_ref() {
            struct_ser.serialize_field("chain", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetChainDescriptorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Chain,
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
                            "chain" => Ok(GeneratedField::Chain),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetChainDescriptorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetChainDescriptorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetChainDescriptorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Chain => {
                            if chain__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chain"));
                            }
                            chain__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetChainDescriptorResponse {
                    chain: chain__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetChainDescriptorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetCodecDescriptorRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetCodecDescriptorRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetCodecDescriptorRequest {
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
            type Value = GetCodecDescriptorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetCodecDescriptorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetCodecDescriptorRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetCodecDescriptorRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetCodecDescriptorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetCodecDescriptorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.codec.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetCodecDescriptorResponse", len)?;
        if let Some(v) = self.codec.as_ref() {
            struct_ser.serialize_field("codec", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetCodecDescriptorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "codec",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Codec,
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
                            "codec" => Ok(GeneratedField::Codec),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetCodecDescriptorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetCodecDescriptorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetCodecDescriptorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut codec__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Codec => {
                            if codec__.is_some() {
                                return Err(serde::de::Error::duplicate_field("codec"));
                            }
                            codec__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetCodecDescriptorResponse {
                    codec: codec__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetCodecDescriptorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetConfigurationDescriptorRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetConfigurationDescriptorRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetConfigurationDescriptorRequest {
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
            type Value = GetConfigurationDescriptorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetConfigurationDescriptorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetConfigurationDescriptorRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetConfigurationDescriptorRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetConfigurationDescriptorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetConfigurationDescriptorResponse {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetConfigurationDescriptorResponse", len)?;
        if let Some(v) = self.config.as_ref() {
            struct_ser.serialize_field("config", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetConfigurationDescriptorResponse {
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
            type Value = GetConfigurationDescriptorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetConfigurationDescriptorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetConfigurationDescriptorResponse, V::Error>
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
                Ok(GetConfigurationDescriptorResponse {
                    config: config__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetConfigurationDescriptorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetQueryServicesDescriptorRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetQueryServicesDescriptorRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetQueryServicesDescriptorRequest {
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
            type Value = GetQueryServicesDescriptorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetQueryServicesDescriptorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetQueryServicesDescriptorRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetQueryServicesDescriptorRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetQueryServicesDescriptorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetQueryServicesDescriptorResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.queries.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetQueryServicesDescriptorResponse", len)?;
        if let Some(v) = self.queries.as_ref() {
            struct_ser.serialize_field("queries", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetQueryServicesDescriptorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "queries",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Queries,
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
                            "queries" => Ok(GeneratedField::Queries),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetQueryServicesDescriptorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetQueryServicesDescriptorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetQueryServicesDescriptorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut queries__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Queries => {
                            if queries__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queries"));
                            }
                            queries__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetQueryServicesDescriptorResponse {
                    queries: queries__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetQueryServicesDescriptorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetTxDescriptorRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetTxDescriptorRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetTxDescriptorRequest {
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
            type Value = GetTxDescriptorRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetTxDescriptorRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetTxDescriptorRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(GetTxDescriptorRequest {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetTxDescriptorRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GetTxDescriptorResponse {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.GetTxDescriptorResponse", len)?;
        if let Some(v) = self.tx.as_ref() {
            struct_ser.serialize_field("tx", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GetTxDescriptorResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tx",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tx,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GetTxDescriptorResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.GetTxDescriptorResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GetTxDescriptorResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tx__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Tx => {
                            if tx__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tx"));
                            }
                            tx__ = map.next_value()?;
                        }
                    }
                }
                Ok(GetTxDescriptorResponse {
                    tx: tx__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.GetTxDescriptorResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InterfaceAcceptingMessageDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fullname.is_empty() {
            len += 1;
        }
        if !self.field_descriptor_names.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.InterfaceAcceptingMessageDescriptor", len)?;
        if !self.fullname.is_empty() {
            struct_ser.serialize_field("fullname", &self.fullname)?;
        }
        if !self.field_descriptor_names.is_empty() {
            struct_ser.serialize_field("fieldDescriptorNames", &self.field_descriptor_names)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InterfaceAcceptingMessageDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fullname",
            "field_descriptor_names",
            "fieldDescriptorNames",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fullname,
            FieldDescriptorNames,
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
                            "fullname" => Ok(GeneratedField::Fullname),
                            "fieldDescriptorNames" | "field_descriptor_names" => Ok(GeneratedField::FieldDescriptorNames),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InterfaceAcceptingMessageDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.InterfaceAcceptingMessageDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InterfaceAcceptingMessageDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fullname__ = None;
                let mut field_descriptor_names__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fullname => {
                            if fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullname"));
                            }
                            fullname__ = Some(map.next_value()?);
                        }
                        GeneratedField::FieldDescriptorNames => {
                            if field_descriptor_names__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fieldDescriptorNames"));
                            }
                            field_descriptor_names__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(InterfaceAcceptingMessageDescriptor {
                    fullname: fullname__.unwrap_or_default(),
                    field_descriptor_names: field_descriptor_names__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.InterfaceAcceptingMessageDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InterfaceDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fullname.is_empty() {
            len += 1;
        }
        if !self.interface_accepting_messages.is_empty() {
            len += 1;
        }
        if !self.interface_implementers.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.InterfaceDescriptor", len)?;
        if !self.fullname.is_empty() {
            struct_ser.serialize_field("fullname", &self.fullname)?;
        }
        if !self.interface_accepting_messages.is_empty() {
            struct_ser.serialize_field("interfaceAcceptingMessages", &self.interface_accepting_messages)?;
        }
        if !self.interface_implementers.is_empty() {
            struct_ser.serialize_field("interfaceImplementers", &self.interface_implementers)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InterfaceDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fullname",
            "interface_accepting_messages",
            "interfaceAcceptingMessages",
            "interface_implementers",
            "interfaceImplementers",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fullname,
            InterfaceAcceptingMessages,
            InterfaceImplementers,
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
                            "fullname" => Ok(GeneratedField::Fullname),
                            "interfaceAcceptingMessages" | "interface_accepting_messages" => Ok(GeneratedField::InterfaceAcceptingMessages),
                            "interfaceImplementers" | "interface_implementers" => Ok(GeneratedField::InterfaceImplementers),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InterfaceDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.InterfaceDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InterfaceDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fullname__ = None;
                let mut interface_accepting_messages__ = None;
                let mut interface_implementers__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fullname => {
                            if fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullname"));
                            }
                            fullname__ = Some(map.next_value()?);
                        }
                        GeneratedField::InterfaceAcceptingMessages => {
                            if interface_accepting_messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interfaceAcceptingMessages"));
                            }
                            interface_accepting_messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::InterfaceImplementers => {
                            if interface_implementers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("interfaceImplementers"));
                            }
                            interface_implementers__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(InterfaceDescriptor {
                    fullname: fullname__.unwrap_or_default(),
                    interface_accepting_messages: interface_accepting_messages__.unwrap_or_default(),
                    interface_implementers: interface_implementers__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.InterfaceDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InterfaceImplementerDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fullname.is_empty() {
            len += 1;
        }
        if !self.type_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.InterfaceImplementerDescriptor", len)?;
        if !self.fullname.is_empty() {
            struct_ser.serialize_field("fullname", &self.fullname)?;
        }
        if !self.type_url.is_empty() {
            struct_ser.serialize_field("typeUrl", &self.type_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for InterfaceImplementerDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fullname",
            "type_url",
            "typeUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fullname,
            TypeUrl,
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
                            "fullname" => Ok(GeneratedField::Fullname),
                            "typeUrl" | "type_url" => Ok(GeneratedField::TypeUrl),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = InterfaceImplementerDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.InterfaceImplementerDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<InterfaceImplementerDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fullname__ = None;
                let mut type_url__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fullname => {
                            if fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullname"));
                            }
                            fullname__ = Some(map.next_value()?);
                        }
                        GeneratedField::TypeUrl => {
                            if type_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("typeUrl"));
                            }
                            type_url__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(InterfaceImplementerDescriptor {
                    fullname: fullname__.unwrap_or_default(),
                    type_url: type_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.InterfaceImplementerDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.msg_type_url.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.MsgDescriptor", len)?;
        if !self.msg_type_url.is_empty() {
            struct_ser.serialize_field("msgTypeUrl", &self.msg_type_url)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "msg_type_url",
            "msgTypeUrl",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            MsgTypeUrl,
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
                            "msgTypeUrl" | "msg_type_url" => Ok(GeneratedField::MsgTypeUrl),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.MsgDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut msg_type_url__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::MsgTypeUrl => {
                            if msg_type_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("msgTypeUrl"));
                            }
                            msg_type_url__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgDescriptor {
                    msg_type_url: msg_type_url__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.MsgDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryMethodDescriptor {
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
        if !self.full_query_path.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.QueryMethodDescriptor", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.full_query_path.is_empty() {
            struct_ser.serialize_field("fullQueryPath", &self.full_query_path)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryMethodDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "full_query_path",
            "fullQueryPath",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            FullQueryPath,
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
                            "fullQueryPath" | "full_query_path" => Ok(GeneratedField::FullQueryPath),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryMethodDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.QueryMethodDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryMethodDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut full_query_path__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::FullQueryPath => {
                            if full_query_path__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullQueryPath"));
                            }
                            full_query_path__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryMethodDescriptor {
                    name: name__.unwrap_or_default(),
                    full_query_path: full_query_path__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.QueryMethodDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryServiceDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fullname.is_empty() {
            len += 1;
        }
        if self.is_module {
            len += 1;
        }
        if !self.methods.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.QueryServiceDescriptor", len)?;
        if !self.fullname.is_empty() {
            struct_ser.serialize_field("fullname", &self.fullname)?;
        }
        if self.is_module {
            struct_ser.serialize_field("isModule", &self.is_module)?;
        }
        if !self.methods.is_empty() {
            struct_ser.serialize_field("methods", &self.methods)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryServiceDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fullname",
            "is_module",
            "isModule",
            "methods",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fullname,
            IsModule,
            Methods,
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
                            "fullname" => Ok(GeneratedField::Fullname),
                            "isModule" | "is_module" => Ok(GeneratedField::IsModule),
                            "methods" => Ok(GeneratedField::Methods),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryServiceDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.QueryServiceDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryServiceDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fullname__ = None;
                let mut is_module__ = None;
                let mut methods__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fullname => {
                            if fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullname"));
                            }
                            fullname__ = Some(map.next_value()?);
                        }
                        GeneratedField::IsModule => {
                            if is_module__.is_some() {
                                return Err(serde::de::Error::duplicate_field("isModule"));
                            }
                            is_module__ = Some(map.next_value()?);
                        }
                        GeneratedField::Methods => {
                            if methods__.is_some() {
                                return Err(serde::de::Error::duplicate_field("methods"));
                            }
                            methods__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryServiceDescriptor {
                    fullname: fullname__.unwrap_or_default(),
                    is_module: is_module__.unwrap_or_default(),
                    methods: methods__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.QueryServiceDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryServicesDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.query_services.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.QueryServicesDescriptor", len)?;
        if !self.query_services.is_empty() {
            struct_ser.serialize_field("queryServices", &self.query_services)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryServicesDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "query_services",
            "queryServices",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            QueryServices,
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
                            "queryServices" | "query_services" => Ok(GeneratedField::QueryServices),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryServicesDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.QueryServicesDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryServicesDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut query_services__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::QueryServices => {
                            if query_services__.is_some() {
                                return Err(serde::de::Error::duplicate_field("queryServices"));
                            }
                            query_services__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryServicesDescriptor {
                    query_services: query_services__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.QueryServicesDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SigningModeDescriptor {
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
        if self.number != 0 {
            len += 1;
        }
        if !self.authn_info_provider_method_fullname.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.SigningModeDescriptor", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if self.number != 0 {
            struct_ser.serialize_field("number", &self.number)?;
        }
        if !self.authn_info_provider_method_fullname.is_empty() {
            struct_ser.serialize_field("authnInfoProviderMethodFullname", &self.authn_info_provider_method_fullname)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SigningModeDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "number",
            "authn_info_provider_method_fullname",
            "authnInfoProviderMethodFullname",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Number,
            AuthnInfoProviderMethodFullname,
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
                            "number" => Ok(GeneratedField::Number),
                            "authnInfoProviderMethodFullname" | "authn_info_provider_method_fullname" => Ok(GeneratedField::AuthnInfoProviderMethodFullname),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SigningModeDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.SigningModeDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SigningModeDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut number__ = None;
                let mut authn_info_provider_method_fullname__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map.next_value()?);
                        }
                        GeneratedField::Number => {
                            if number__.is_some() {
                                return Err(serde::de::Error::duplicate_field("number"));
                            }
                            number__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::AuthnInfoProviderMethodFullname => {
                            if authn_info_provider_method_fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("authnInfoProviderMethodFullname"));
                            }
                            authn_info_provider_method_fullname__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SigningModeDescriptor {
                    name: name__.unwrap_or_default(),
                    number: number__.unwrap_or_default(),
                    authn_info_provider_method_fullname: authn_info_provider_method_fullname__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.SigningModeDescriptor", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TxDescriptor {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.fullname.is_empty() {
            len += 1;
        }
        if !self.msgs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.base.reflection.v2alpha1.TxDescriptor", len)?;
        if !self.fullname.is_empty() {
            struct_ser.serialize_field("fullname", &self.fullname)?;
        }
        if !self.msgs.is_empty() {
            struct_ser.serialize_field("msgs", &self.msgs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TxDescriptor {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "fullname",
            "msgs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Fullname,
            Msgs,
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
                            "fullname" => Ok(GeneratedField::Fullname),
                            "msgs" => Ok(GeneratedField::Msgs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TxDescriptor;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.base.reflection.v2alpha1.TxDescriptor")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TxDescriptor, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut fullname__ = None;
                let mut msgs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Fullname => {
                            if fullname__.is_some() {
                                return Err(serde::de::Error::duplicate_field("fullname"));
                            }
                            fullname__ = Some(map.next_value()?);
                        }
                        GeneratedField::Msgs => {
                            if msgs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("msgs"));
                            }
                            msgs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TxDescriptor {
                    fullname: fullname__.unwrap_or_default(),
                    msgs: msgs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.base.reflection.v2alpha1.TxDescriptor", FIELDS, GeneratedVisitor)
    }
}
