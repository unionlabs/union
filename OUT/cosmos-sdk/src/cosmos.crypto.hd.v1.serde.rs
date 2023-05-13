// @generated
impl serde::Serialize for Bip44Params {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.purpose != 0 {
            len += 1;
        }
        if self.coin_type != 0 {
            len += 1;
        }
        if self.account != 0 {
            len += 1;
        }
        if self.change {
            len += 1;
        }
        if self.address_index != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.crypto.hd.v1.BIP44Params", len)?;
        if self.purpose != 0 {
            struct_ser.serialize_field("purpose", &self.purpose)?;
        }
        if self.coin_type != 0 {
            struct_ser.serialize_field("coinType", &self.coin_type)?;
        }
        if self.account != 0 {
            struct_ser.serialize_field("account", &self.account)?;
        }
        if self.change {
            struct_ser.serialize_field("change", &self.change)?;
        }
        if self.address_index != 0 {
            struct_ser.serialize_field("addressIndex", &self.address_index)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Bip44Params {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "purpose",
            "coin_type",
            "coinType",
            "account",
            "change",
            "address_index",
            "addressIndex",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Purpose,
            CoinType,
            Account,
            Change,
            AddressIndex,
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
                            "purpose" => Ok(GeneratedField::Purpose),
                            "coinType" | "coin_type" => Ok(GeneratedField::CoinType),
                            "account" => Ok(GeneratedField::Account),
                            "change" => Ok(GeneratedField::Change),
                            "addressIndex" | "address_index" => Ok(GeneratedField::AddressIndex),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Bip44Params;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.crypto.hd.v1.BIP44Params")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Bip44Params, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut purpose__ = None;
                let mut coin_type__ = None;
                let mut account__ = None;
                let mut change__ = None;
                let mut address_index__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Purpose => {
                            if purpose__.is_some() {
                                return Err(serde::de::Error::duplicate_field("purpose"));
                            }
                            purpose__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::CoinType => {
                            if coin_type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("coinType"));
                            }
                            coin_type__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Account => {
                            if account__.is_some() {
                                return Err(serde::de::Error::duplicate_field("account"));
                            }
                            account__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Change => {
                            if change__.is_some() {
                                return Err(serde::de::Error::duplicate_field("change"));
                            }
                            change__ = Some(map.next_value()?);
                        }
                        GeneratedField::AddressIndex => {
                            if address_index__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addressIndex"));
                            }
                            address_index__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(Bip44Params {
                    purpose: purpose__.unwrap_or_default(),
                    coin_type: coin_type__.unwrap_or_default(),
                    account: account__.unwrap_or_default(),
                    change: change__.unwrap_or_default(),
                    address_index: address_index__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.crypto.hd.v1.BIP44Params", FIELDS, GeneratedVisitor)
    }
}
