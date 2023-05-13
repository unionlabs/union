// @generated
impl serde::Serialize for Errors {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unknown => "ERRORS_UNKNOWN",
            Self::UnexpectedResponse => "ERRORS_UNEXPECTED_RESPONSE",
            Self::NoConnection => "ERRORS_NO_CONNECTION",
            Self::ConnectionTimeout => "ERRORS_CONNECTION_TIMEOUT",
            Self::ReadTimeout => "ERRORS_READ_TIMEOUT",
            Self::WriteTimeout => "ERRORS_WRITE_TIMEOUT",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Errors {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "ERRORS_UNKNOWN",
            "ERRORS_UNEXPECTED_RESPONSE",
            "ERRORS_NO_CONNECTION",
            "ERRORS_CONNECTION_TIMEOUT",
            "ERRORS_READ_TIMEOUT",
            "ERRORS_WRITE_TIMEOUT",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Errors;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "expected one of: {:?}", &FIELDS)
            }

            fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(Errors::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Signed(v), &self)
                    })
            }

            fn visit_u64<E>(self, v: u64) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::convert::TryFrom;
                i32::try_from(v)
                    .ok()
                    .and_then(Errors::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "ERRORS_UNKNOWN" => Ok(Errors::Unknown),
                    "ERRORS_UNEXPECTED_RESPONSE" => Ok(Errors::UnexpectedResponse),
                    "ERRORS_NO_CONNECTION" => Ok(Errors::NoConnection),
                    "ERRORS_CONNECTION_TIMEOUT" => Ok(Errors::ConnectionTimeout),
                    "ERRORS_READ_TIMEOUT" => Ok(Errors::ReadTimeout),
                    "ERRORS_WRITE_TIMEOUT" => Ok(Errors::WriteTimeout),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for Message {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.sum.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.Message", len)?;
        if let Some(v) = self.sum.as_ref() {
            match v {
                message::Sum::PubKeyRequest(v) => {
                    struct_ser.serialize_field("pubKeyRequest", v)?;
                }
                message::Sum::PubKeyResponse(v) => {
                    struct_ser.serialize_field("pubKeyResponse", v)?;
                }
                message::Sum::SignVoteRequest(v) => {
                    struct_ser.serialize_field("signVoteRequest", v)?;
                }
                message::Sum::SignedVoteResponse(v) => {
                    struct_ser.serialize_field("signedVoteResponse", v)?;
                }
                message::Sum::SignProposalRequest(v) => {
                    struct_ser.serialize_field("signProposalRequest", v)?;
                }
                message::Sum::SignedProposalResponse(v) => {
                    struct_ser.serialize_field("signedProposalResponse", v)?;
                }
                message::Sum::PingRequest(v) => {
                    struct_ser.serialize_field("pingRequest", v)?;
                }
                message::Sum::PingResponse(v) => {
                    struct_ser.serialize_field("pingResponse", v)?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Message {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pub_key_request",
            "pubKeyRequest",
            "pub_key_response",
            "pubKeyResponse",
            "sign_vote_request",
            "signVoteRequest",
            "signed_vote_response",
            "signedVoteResponse",
            "sign_proposal_request",
            "signProposalRequest",
            "signed_proposal_response",
            "signedProposalResponse",
            "ping_request",
            "pingRequest",
            "ping_response",
            "pingResponse",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PubKeyRequest,
            PubKeyResponse,
            SignVoteRequest,
            SignedVoteResponse,
            SignProposalRequest,
            SignedProposalResponse,
            PingRequest,
            PingResponse,
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
                            "pubKeyRequest" | "pub_key_request" => Ok(GeneratedField::PubKeyRequest),
                            "pubKeyResponse" | "pub_key_response" => Ok(GeneratedField::PubKeyResponse),
                            "signVoteRequest" | "sign_vote_request" => Ok(GeneratedField::SignVoteRequest),
                            "signedVoteResponse" | "signed_vote_response" => Ok(GeneratedField::SignedVoteResponse),
                            "signProposalRequest" | "sign_proposal_request" => Ok(GeneratedField::SignProposalRequest),
                            "signedProposalResponse" | "signed_proposal_response" => Ok(GeneratedField::SignedProposalResponse),
                            "pingRequest" | "ping_request" => Ok(GeneratedField::PingRequest),
                            "pingResponse" | "ping_response" => Ok(GeneratedField::PingResponse),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Message;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.Message")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Message, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut sum__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PubKeyRequest => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKeyRequest"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::PubKeyRequest)
;
                        }
                        GeneratedField::PubKeyResponse => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKeyResponse"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::PubKeyResponse)
;
                        }
                        GeneratedField::SignVoteRequest => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signVoteRequest"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::SignVoteRequest)
;
                        }
                        GeneratedField::SignedVoteResponse => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedVoteResponse"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::SignedVoteResponse)
;
                        }
                        GeneratedField::SignProposalRequest => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signProposalRequest"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::SignProposalRequest)
;
                        }
                        GeneratedField::SignedProposalResponse => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("signedProposalResponse"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::SignedProposalResponse)
;
                        }
                        GeneratedField::PingRequest => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pingRequest"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::PingRequest)
;
                        }
                        GeneratedField::PingResponse => {
                            if sum__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pingResponse"));
                            }
                            sum__ = map.next_value::<::std::option::Option<_>>()?.map(message::Sum::PingResponse)
;
                        }
                    }
                }
                Ok(Message {
                    sum: sum__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.Message", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PingRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("tendermint.privval.PingRequest", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PingRequest {
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
            type Value = PingRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.PingRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PingRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PingRequest {
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.PingRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PingResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("tendermint.privval.PingResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PingResponse {
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
            type Value = PingResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.PingResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PingResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(PingResponse {
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.PingResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PubKeyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.chain_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.PubKeyRequest", len)?;
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PubKeyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "chain_id",
            "chainId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ChainId,
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
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PubKeyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.PubKeyRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PubKeyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut chain_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(PubKeyRequest {
                    chain_id: chain_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.PubKeyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PubKeyResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.pub_key.is_some() {
            len += 1;
        }
        if self.error.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.PubKeyResponse", len)?;
        if let Some(v) = self.pub_key.as_ref() {
            struct_ser.serialize_field("pubKey", v)?;
        }
        if let Some(v) = self.error.as_ref() {
            struct_ser.serialize_field("error", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PubKeyResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "pub_key",
            "pubKey",
            "error",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            PubKey,
            Error,
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
                            "pubKey" | "pub_key" => Ok(GeneratedField::PubKey),
                            "error" => Ok(GeneratedField::Error),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PubKeyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.PubKeyResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PubKeyResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut pub_key__ = None;
                let mut error__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::PubKey => {
                            if pub_key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pubKey"));
                            }
                            pub_key__ = map.next_value()?;
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = map.next_value()?;
                        }
                    }
                }
                Ok(PubKeyResponse {
                    pub_key: pub_key__,
                    error: error__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.PubKeyResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for RemoteSignerError {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.code != 0 {
            len += 1;
        }
        if !self.description.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.RemoteSignerError", len)?;
        if self.code != 0 {
            struct_ser.serialize_field("code", &self.code)?;
        }
        if !self.description.is_empty() {
            struct_ser.serialize_field("description", &self.description)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for RemoteSignerError {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "code",
            "description",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Code,
            Description,
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
                            "code" => Ok(GeneratedField::Code),
                            "description" => Ok(GeneratedField::Description),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = RemoteSignerError;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.RemoteSignerError")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<RemoteSignerError, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut code__ = None;
                let mut description__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Code => {
                            if code__.is_some() {
                                return Err(serde::de::Error::duplicate_field("code"));
                            }
                            code__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Description => {
                            if description__.is_some() {
                                return Err(serde::de::Error::duplicate_field("description"));
                            }
                            description__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(RemoteSignerError {
                    code: code__.unwrap_or_default(),
                    description: description__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.RemoteSignerError", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignProposalRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal.is_some() {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.SignProposalRequest", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignProposalRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "chain_id",
            "chainId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            ChainId,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignProposalRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.SignProposalRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SignProposalRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut chain_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map.next_value()?;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SignProposalRequest {
                    proposal: proposal__,
                    chain_id: chain_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.SignProposalRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignVoteRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote.is_some() {
            len += 1;
        }
        if !self.chain_id.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.SignVoteRequest", len)?;
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if !self.chain_id.is_empty() {
            struct_ser.serialize_field("chainId", &self.chain_id)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignVoteRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
            "chain_id",
            "chainId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
            ChainId,
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
                            "vote" => Ok(GeneratedField::Vote),
                            "chainId" | "chain_id" => Ok(GeneratedField::ChainId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignVoteRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.SignVoteRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SignVoteRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                let mut chain_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map.next_value()?;
                        }
                        GeneratedField::ChainId => {
                            if chain_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("chainId"));
                            }
                            chain_id__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(SignVoteRequest {
                    vote: vote__,
                    chain_id: chain_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.SignVoteRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignedProposalResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal.is_some() {
            len += 1;
        }
        if self.error.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.SignedProposalResponse", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        if let Some(v) = self.error.as_ref() {
            struct_ser.serialize_field("error", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignedProposalResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
            "error",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
            Error,
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
                            "proposal" => Ok(GeneratedField::Proposal),
                            "error" => Ok(GeneratedField::Error),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignedProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.SignedProposalResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SignedProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                let mut error__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map.next_value()?;
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = map.next_value()?;
                        }
                    }
                }
                Ok(SignedProposalResponse {
                    proposal: proposal__,
                    error: error__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.SignedProposalResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for SignedVoteResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.vote.is_some() {
            len += 1;
        }
        if self.error.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("tendermint.privval.SignedVoteResponse", len)?;
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        if let Some(v) = self.error.as_ref() {
            struct_ser.serialize_field("error", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for SignedVoteResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
            "error",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
            Error,
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
                            "vote" => Ok(GeneratedField::Vote),
                            "error" => Ok(GeneratedField::Error),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = SignedVoteResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct tendermint.privval.SignedVoteResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<SignedVoteResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                let mut error__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map.next_value()?;
                        }
                        GeneratedField::Error => {
                            if error__.is_some() {
                                return Err(serde::de::Error::duplicate_field("error"));
                            }
                            error__ = map.next_value()?;
                        }
                    }
                }
                Ok(SignedVoteResponse {
                    vote: vote__,
                    error: error__,
                })
            }
        }
        deserializer.deserialize_struct("tendermint.privval.SignedVoteResponse", FIELDS, GeneratedVisitor)
    }
}
