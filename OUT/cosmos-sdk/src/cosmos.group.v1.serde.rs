// @generated
impl serde::Serialize for DecisionPolicyWindows {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.voting_period.is_some() {
            len += 1;
        }
        if self.min_execution_period.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.DecisionPolicyWindows", len)?;
        if let Some(v) = self.voting_period.as_ref() {
            struct_ser.serialize_field("votingPeriod", v)?;
        }
        if let Some(v) = self.min_execution_period.as_ref() {
            struct_ser.serialize_field("minExecutionPeriod", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for DecisionPolicyWindows {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "voting_period",
            "votingPeriod",
            "min_execution_period",
            "minExecutionPeriod",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            VotingPeriod,
            MinExecutionPeriod,
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
                            "votingPeriod" | "voting_period" => Ok(GeneratedField::VotingPeriod),
                            "minExecutionPeriod" | "min_execution_period" => Ok(GeneratedField::MinExecutionPeriod),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = DecisionPolicyWindows;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.DecisionPolicyWindows")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<DecisionPolicyWindows, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut voting_period__ = None;
                let mut min_execution_period__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::VotingPeriod => {
                            if voting_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPeriod"));
                            }
                            voting_period__ = map.next_value()?;
                        }
                        GeneratedField::MinExecutionPeriod => {
                            if min_execution_period__.is_some() {
                                return Err(serde::de::Error::duplicate_field("minExecutionPeriod"));
                            }
                            min_execution_period__ = map.next_value()?;
                        }
                    }
                }
                Ok(DecisionPolicyWindows {
                    voting_period: voting_period__,
                    min_execution_period: min_execution_period__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.DecisionPolicyWindows", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventCreateGroup {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventCreateGroup", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventCreateGroup {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventCreateGroup;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventCreateGroup")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventCreateGroup, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EventCreateGroup {
                    group_id: group_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventCreateGroup", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventCreateGroupPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventCreateGroupPolicy", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventCreateGroupPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
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
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventCreateGroupPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventCreateGroupPolicy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventCreateGroupPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(EventCreateGroupPolicy {
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventCreateGroupPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventExec {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if self.result != 0 {
            len += 1;
        }
        if !self.logs.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventExec", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if self.result != 0 {
            let v = ProposalExecutorResult::from_i32(self.result)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.result)))?;
            struct_ser.serialize_field("result", &v)?;
        }
        if !self.logs.is_empty() {
            struct_ser.serialize_field("logs", &self.logs)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventExec {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "result",
            "logs",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Result,
            Logs,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "result" => Ok(GeneratedField::Result),
                            "logs" => Ok(GeneratedField::Logs),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventExec;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventExec")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventExec, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut result__ = None;
                let mut logs__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Result => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result__ = Some(map.next_value::<ProposalExecutorResult>()? as i32);
                        }
                        GeneratedField::Logs => {
                            if logs__.is_some() {
                                return Err(serde::de::Error::duplicate_field("logs"));
                            }
                            logs__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(EventExec {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    result: result__.unwrap_or_default(),
                    logs: logs__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventExec", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventLeaveGroup {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        if !self.address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventLeaveGroup", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventLeaveGroup {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
            Address,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventLeaveGroup;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventLeaveGroup")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventLeaveGroup, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(EventLeaveGroup {
                    group_id: group_id__.unwrap_or_default(),
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventLeaveGroup", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventSubmitProposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventSubmitProposal", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventSubmitProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventSubmitProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventSubmitProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventSubmitProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EventSubmitProposal {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventSubmitProposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventUpdateGroup {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventUpdateGroup", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventUpdateGroup {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventUpdateGroup;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventUpdateGroup")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventUpdateGroup, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EventUpdateGroup {
                    group_id: group_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventUpdateGroup", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventUpdateGroupPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventUpdateGroupPolicy", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventUpdateGroupPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
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
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventUpdateGroupPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventUpdateGroupPolicy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventUpdateGroupPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(EventUpdateGroupPolicy {
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventUpdateGroupPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventVote", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventVote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EventVote {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EventWithdrawProposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.EventWithdrawProposal", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EventWithdrawProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EventWithdrawProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.EventWithdrawProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<EventWithdrawProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(EventWithdrawProposal {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.EventWithdrawProposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Exec {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "EXEC_UNSPECIFIED",
            Self::Try => "EXEC_TRY",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for Exec {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "EXEC_UNSPECIFIED",
            "EXEC_TRY",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Exec;

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
                    .and_then(Exec::from_i32)
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
                    .and_then(Exec::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "EXEC_UNSPECIFIED" => Ok(Exec::Unspecified),
                    "EXEC_TRY" => Ok(Exec::Try),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for GenesisState {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_seq != 0 {
            len += 1;
        }
        if !self.groups.is_empty() {
            len += 1;
        }
        if !self.group_members.is_empty() {
            len += 1;
        }
        if self.group_policy_seq != 0 {
            len += 1;
        }
        if !self.group_policies.is_empty() {
            len += 1;
        }
        if self.proposal_seq != 0 {
            len += 1;
        }
        if !self.proposals.is_empty() {
            len += 1;
        }
        if !self.votes.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.GenesisState", len)?;
        if self.group_seq != 0 {
            struct_ser.serialize_field("groupSeq", ToString::to_string(&self.group_seq).as_str())?;
        }
        if !self.groups.is_empty() {
            struct_ser.serialize_field("groups", &self.groups)?;
        }
        if !self.group_members.is_empty() {
            struct_ser.serialize_field("groupMembers", &self.group_members)?;
        }
        if self.group_policy_seq != 0 {
            struct_ser.serialize_field("groupPolicySeq", ToString::to_string(&self.group_policy_seq).as_str())?;
        }
        if !self.group_policies.is_empty() {
            struct_ser.serialize_field("groupPolicies", &self.group_policies)?;
        }
        if self.proposal_seq != 0 {
            struct_ser.serialize_field("proposalSeq", ToString::to_string(&self.proposal_seq).as_str())?;
        }
        if !self.proposals.is_empty() {
            struct_ser.serialize_field("proposals", &self.proposals)?;
        }
        if !self.votes.is_empty() {
            struct_ser.serialize_field("votes", &self.votes)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GenesisState {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_seq",
            "groupSeq",
            "groups",
            "group_members",
            "groupMembers",
            "group_policy_seq",
            "groupPolicySeq",
            "group_policies",
            "groupPolicies",
            "proposal_seq",
            "proposalSeq",
            "proposals",
            "votes",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupSeq,
            Groups,
            GroupMembers,
            GroupPolicySeq,
            GroupPolicies,
            ProposalSeq,
            Proposals,
            Votes,
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
                            "groupSeq" | "group_seq" => Ok(GeneratedField::GroupSeq),
                            "groups" => Ok(GeneratedField::Groups),
                            "groupMembers" | "group_members" => Ok(GeneratedField::GroupMembers),
                            "groupPolicySeq" | "group_policy_seq" => Ok(GeneratedField::GroupPolicySeq),
                            "groupPolicies" | "group_policies" => Ok(GeneratedField::GroupPolicies),
                            "proposalSeq" | "proposal_seq" => Ok(GeneratedField::ProposalSeq),
                            "proposals" => Ok(GeneratedField::Proposals),
                            "votes" => Ok(GeneratedField::Votes),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GenesisState;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.GenesisState")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GenesisState, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_seq__ = None;
                let mut groups__ = None;
                let mut group_members__ = None;
                let mut group_policy_seq__ = None;
                let mut group_policies__ = None;
                let mut proposal_seq__ = None;
                let mut proposals__ = None;
                let mut votes__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupSeq => {
                            if group_seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupSeq"));
                            }
                            group_seq__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Groups => {
                            if groups__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groups"));
                            }
                            groups__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupMembers => {
                            if group_members__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupMembers"));
                            }
                            group_members__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupPolicySeq => {
                            if group_policy_seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicySeq"));
                            }
                            group_policy_seq__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GroupPolicies => {
                            if group_policies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicies"));
                            }
                            group_policies__ = Some(map.next_value()?);
                        }
                        GeneratedField::ProposalSeq => {
                            if proposal_seq__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalSeq"));
                            }
                            proposal_seq__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Proposals => {
                            if proposals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposals"));
                            }
                            proposals__ = Some(map.next_value()?);
                        }
                        GeneratedField::Votes => {
                            if votes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votes"));
                            }
                            votes__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(GenesisState {
                    group_seq: group_seq__.unwrap_or_default(),
                    groups: groups__.unwrap_or_default(),
                    group_members: group_members__.unwrap_or_default(),
                    group_policy_seq: group_policy_seq__.unwrap_or_default(),
                    group_policies: group_policies__.unwrap_or_default(),
                    proposal_seq: proposal_seq__.unwrap_or_default(),
                    proposals: proposals__.unwrap_or_default(),
                    votes: votes__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.GenesisState", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GroupInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.admin.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.version != 0 {
            len += 1;
        }
        if !self.total_weight.is_empty() {
            len += 1;
        }
        if self.created_at.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.GroupInfo", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", ToString::to_string(&self.id).as_str())?;
        }
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if self.version != 0 {
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        if !self.total_weight.is_empty() {
            struct_ser.serialize_field("totalWeight", &self.total_weight)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GroupInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "admin",
            "metadata",
            "version",
            "total_weight",
            "totalWeight",
            "created_at",
            "createdAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            Admin,
            Metadata,
            Version,
            TotalWeight,
            CreatedAt,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "version" => Ok(GeneratedField::Version),
                            "totalWeight" | "total_weight" => Ok(GeneratedField::TotalWeight),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GroupInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.GroupInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GroupInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut admin__ = None;
                let mut metadata__ = None;
                let mut version__ = None;
                let mut total_weight__ = None;
                let mut created_at__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::TotalWeight => {
                            if total_weight__.is_some() {
                                return Err(serde::de::Error::duplicate_field("totalWeight"));
                            }
                            total_weight__ = Some(map.next_value()?);
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map.next_value()?;
                        }
                    }
                }
                Ok(GroupInfo {
                    id: id__.unwrap_or_default(),
                    admin: admin__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    total_weight: total_weight__.unwrap_or_default(),
                    created_at: created_at__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.GroupInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GroupMember {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        if self.member.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.GroupMember", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if let Some(v) = self.member.as_ref() {
            struct_ser.serialize_field("member", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GroupMember {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
            "member",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
            Member,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "member" => Ok(GeneratedField::Member),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GroupMember;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.GroupMember")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GroupMember, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                let mut member__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Member => {
                            if member__.is_some() {
                                return Err(serde::de::Error::duplicate_field("member"));
                            }
                            member__ = map.next_value()?;
                        }
                    }
                }
                Ok(GroupMember {
                    group_id: group_id__.unwrap_or_default(),
                    member: member__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.GroupMember", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for GroupPolicyInfo {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if self.group_id != 0 {
            len += 1;
        }
        if !self.admin.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.version != 0 {
            len += 1;
        }
        if self.decision_policy.is_some() {
            len += 1;
        }
        if self.created_at.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.GroupPolicyInfo", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if self.version != 0 {
            struct_ser.serialize_field("version", ToString::to_string(&self.version).as_str())?;
        }
        if let Some(v) = self.decision_policy.as_ref() {
            struct_ser.serialize_field("decisionPolicy", v)?;
        }
        if let Some(v) = self.created_at.as_ref() {
            struct_ser.serialize_field("createdAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for GroupPolicyInfo {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "group_id",
            "groupId",
            "admin",
            "metadata",
            "version",
            "decision_policy",
            "decisionPolicy",
            "created_at",
            "createdAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            GroupId,
            Admin,
            Metadata,
            Version,
            DecisionPolicy,
            CreatedAt,
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
                            "address" => Ok(GeneratedField::Address),
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "admin" => Ok(GeneratedField::Admin),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "version" => Ok(GeneratedField::Version),
                            "decisionPolicy" | "decision_policy" => Ok(GeneratedField::DecisionPolicy),
                            "createdAt" | "created_at" => Ok(GeneratedField::CreatedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = GroupPolicyInfo;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.GroupPolicyInfo")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<GroupPolicyInfo, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut group_id__ = None;
                let mut admin__ = None;
                let mut metadata__ = None;
                let mut version__ = None;
                let mut decision_policy__ = None;
                let mut created_at__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::DecisionPolicy => {
                            if decision_policy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("decisionPolicy"));
                            }
                            decision_policy__ = map.next_value()?;
                        }
                        GeneratedField::CreatedAt => {
                            if created_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("createdAt"));
                            }
                            created_at__ = map.next_value()?;
                        }
                    }
                }
                Ok(GroupPolicyInfo {
                    address: address__.unwrap_or_default(),
                    group_id: group_id__.unwrap_or_default(),
                    admin: admin__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    decision_policy: decision_policy__,
                    created_at: created_at__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.GroupPolicyInfo", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Member {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if !self.weight.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.added_at.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.Member", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if !self.weight.is_empty() {
            struct_ser.serialize_field("weight", &self.weight)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if let Some(v) = self.added_at.as_ref() {
            struct_ser.serialize_field("addedAt", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Member {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "weight",
            "metadata",
            "added_at",
            "addedAt",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Weight,
            Metadata,
            AddedAt,
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
                            "address" => Ok(GeneratedField::Address),
                            "weight" => Ok(GeneratedField::Weight),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "addedAt" | "added_at" => Ok(GeneratedField::AddedAt),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Member;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.Member")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Member, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut weight__ = None;
                let mut metadata__ = None;
                let mut added_at__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Weight => {
                            if weight__.is_some() {
                                return Err(serde::de::Error::duplicate_field("weight"));
                            }
                            weight__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::AddedAt => {
                            if added_at__.is_some() {
                                return Err(serde::de::Error::duplicate_field("addedAt"));
                            }
                            added_at__ = map.next_value()?;
                        }
                    }
                }
                Ok(Member {
                    address: address__.unwrap_or_default(),
                    weight: weight__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    added_at: added_at__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.Member", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MemberRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if !self.weight.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MemberRequest", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if !self.weight.is_empty() {
            struct_ser.serialize_field("weight", &self.weight)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MemberRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "weight",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Weight,
            Metadata,
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
                            "address" => Ok(GeneratedField::Address),
                            "weight" => Ok(GeneratedField::Weight),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MemberRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MemberRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MemberRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut weight__ = None;
                let mut metadata__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Weight => {
                            if weight__.is_some() {
                                return Err(serde::de::Error::duplicate_field("weight"));
                            }
                            weight__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MemberRequest {
                    address: address__.unwrap_or_default(),
                    weight: weight__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MemberRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateGroup {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if !self.members.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgCreateGroup", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if !self.members.is_empty() {
            struct_ser.serialize_field("members", &self.members)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateGroup {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "members",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            Members,
            Metadata,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "members" => Ok(GeneratedField::Members),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateGroup;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgCreateGroup")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateGroup, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut members__ = None;
                let mut metadata__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::Members => {
                            if members__.is_some() {
                                return Err(serde::de::Error::duplicate_field("members"));
                            }
                            members__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgCreateGroup {
                    admin: admin__.unwrap_or_default(),
                    members: members__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgCreateGroup", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateGroupPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if self.group_id != 0 {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.decision_policy.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgCreateGroupPolicy", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if let Some(v) = self.decision_policy.as_ref() {
            struct_ser.serialize_field("decisionPolicy", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateGroupPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "group_id",
            "groupId",
            "metadata",
            "decision_policy",
            "decisionPolicy",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            GroupId,
            Metadata,
            DecisionPolicy,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "decisionPolicy" | "decision_policy" => Ok(GeneratedField::DecisionPolicy),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateGroupPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgCreateGroupPolicy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateGroupPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut group_id__ = None;
                let mut metadata__ = None;
                let mut decision_policy__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::DecisionPolicy => {
                            if decision_policy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("decisionPolicy"));
                            }
                            decision_policy__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgCreateGroupPolicy {
                    admin: admin__.unwrap_or_default(),
                    group_id: group_id__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    decision_policy: decision_policy__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgCreateGroupPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateGroupPolicyResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgCreateGroupPolicyResponse", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateGroupPolicyResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
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
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateGroupPolicyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgCreateGroupPolicyResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateGroupPolicyResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgCreateGroupPolicyResponse {
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgCreateGroupPolicyResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateGroupResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgCreateGroupResponse", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateGroupResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateGroupResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgCreateGroupResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateGroupResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgCreateGroupResponse {
                    group_id: group_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgCreateGroupResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateGroupWithPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if !self.members.is_empty() {
            len += 1;
        }
        if !self.group_metadata.is_empty() {
            len += 1;
        }
        if !self.group_policy_metadata.is_empty() {
            len += 1;
        }
        if self.group_policy_as_admin {
            len += 1;
        }
        if self.decision_policy.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgCreateGroupWithPolicy", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if !self.members.is_empty() {
            struct_ser.serialize_field("members", &self.members)?;
        }
        if !self.group_metadata.is_empty() {
            struct_ser.serialize_field("groupMetadata", &self.group_metadata)?;
        }
        if !self.group_policy_metadata.is_empty() {
            struct_ser.serialize_field("groupPolicyMetadata", &self.group_policy_metadata)?;
        }
        if self.group_policy_as_admin {
            struct_ser.serialize_field("groupPolicyAsAdmin", &self.group_policy_as_admin)?;
        }
        if let Some(v) = self.decision_policy.as_ref() {
            struct_ser.serialize_field("decisionPolicy", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateGroupWithPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "members",
            "group_metadata",
            "groupMetadata",
            "group_policy_metadata",
            "groupPolicyMetadata",
            "group_policy_as_admin",
            "groupPolicyAsAdmin",
            "decision_policy",
            "decisionPolicy",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            Members,
            GroupMetadata,
            GroupPolicyMetadata,
            GroupPolicyAsAdmin,
            DecisionPolicy,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "members" => Ok(GeneratedField::Members),
                            "groupMetadata" | "group_metadata" => Ok(GeneratedField::GroupMetadata),
                            "groupPolicyMetadata" | "group_policy_metadata" => Ok(GeneratedField::GroupPolicyMetadata),
                            "groupPolicyAsAdmin" | "group_policy_as_admin" => Ok(GeneratedField::GroupPolicyAsAdmin),
                            "decisionPolicy" | "decision_policy" => Ok(GeneratedField::DecisionPolicy),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateGroupWithPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgCreateGroupWithPolicy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateGroupWithPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut members__ = None;
                let mut group_metadata__ = None;
                let mut group_policy_metadata__ = None;
                let mut group_policy_as_admin__ = None;
                let mut decision_policy__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::Members => {
                            if members__.is_some() {
                                return Err(serde::de::Error::duplicate_field("members"));
                            }
                            members__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupMetadata => {
                            if group_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupMetadata"));
                            }
                            group_metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupPolicyMetadata => {
                            if group_policy_metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyMetadata"));
                            }
                            group_policy_metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupPolicyAsAdmin => {
                            if group_policy_as_admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyAsAdmin"));
                            }
                            group_policy_as_admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::DecisionPolicy => {
                            if decision_policy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("decisionPolicy"));
                            }
                            decision_policy__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgCreateGroupWithPolicy {
                    admin: admin__.unwrap_or_default(),
                    members: members__.unwrap_or_default(),
                    group_metadata: group_metadata__.unwrap_or_default(),
                    group_policy_metadata: group_policy_metadata__.unwrap_or_default(),
                    group_policy_as_admin: group_policy_as_admin__.unwrap_or_default(),
                    decision_policy: decision_policy__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgCreateGroupWithPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgCreateGroupWithPolicyResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        if !self.group_policy_address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgCreateGroupWithPolicyResponse", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if !self.group_policy_address.is_empty() {
            struct_ser.serialize_field("groupPolicyAddress", &self.group_policy_address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgCreateGroupWithPolicyResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
            "group_policy_address",
            "groupPolicyAddress",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
            GroupPolicyAddress,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "groupPolicyAddress" | "group_policy_address" => Ok(GeneratedField::GroupPolicyAddress),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgCreateGroupWithPolicyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgCreateGroupWithPolicyResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgCreateGroupWithPolicyResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                let mut group_policy_address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GroupPolicyAddress => {
                            if group_policy_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyAddress"));
                            }
                            group_policy_address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgCreateGroupWithPolicyResponse {
                    group_id: group_id__.unwrap_or_default(),
                    group_policy_address: group_policy_address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgCreateGroupWithPolicyResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgExec {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.executor.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgExec", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.executor.is_empty() {
            struct_ser.serialize_field("executor", &self.executor)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgExec {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "executor",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Executor,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "executor" => Ok(GeneratedField::Executor),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgExec;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgExec")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgExec, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut executor__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Executor => {
                            if executor__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executor"));
                            }
                            executor__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgExec {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    executor: executor__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgExec", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgExecResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.result != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgExecResponse", len)?;
        if self.result != 0 {
            let v = ProposalExecutorResult::from_i32(self.result)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.result)))?;
            struct_ser.serialize_field("result", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgExecResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "result",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Result,
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
                            "result" => Ok(GeneratedField::Result),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgExecResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgExecResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgExecResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut result__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Result => {
                            if result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("result"));
                            }
                            result__ = Some(map.next_value::<ProposalExecutorResult>()? as i32);
                        }
                    }
                }
                Ok(MsgExecResponse {
                    result: result__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgExecResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgLeaveGroup {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if self.group_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgLeaveGroup", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgLeaveGroup {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "group_id",
            "groupId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            GroupId,
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
                            "address" => Ok(GeneratedField::Address),
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgLeaveGroup;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgLeaveGroup")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgLeaveGroup, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut group_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgLeaveGroup {
                    address: address__.unwrap_or_default(),
                    group_id: group_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgLeaveGroup", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgLeaveGroupResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgLeaveGroupResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgLeaveGroupResponse {
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
            type Value = MsgLeaveGroupResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgLeaveGroupResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgLeaveGroupResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgLeaveGroupResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgLeaveGroupResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgSubmitProposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.group_policy_address.is_empty() {
            len += 1;
        }
        if !self.proposers.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if !self.messages.is_empty() {
            len += 1;
        }
        if self.exec != 0 {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.summary.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgSubmitProposal", len)?;
        if !self.group_policy_address.is_empty() {
            struct_ser.serialize_field("groupPolicyAddress", &self.group_policy_address)?;
        }
        if !self.proposers.is_empty() {
            struct_ser.serialize_field("proposers", &self.proposers)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if !self.messages.is_empty() {
            struct_ser.serialize_field("messages", &self.messages)?;
        }
        if self.exec != 0 {
            let v = Exec::from_i32(self.exec)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.exec)))?;
            struct_ser.serialize_field("exec", &v)?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.summary.is_empty() {
            struct_ser.serialize_field("summary", &self.summary)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgSubmitProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_policy_address",
            "groupPolicyAddress",
            "proposers",
            "metadata",
            "messages",
            "exec",
            "title",
            "summary",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupPolicyAddress,
            Proposers,
            Metadata,
            Messages,
            Exec,
            Title,
            Summary,
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
                            "groupPolicyAddress" | "group_policy_address" => Ok(GeneratedField::GroupPolicyAddress),
                            "proposers" => Ok(GeneratedField::Proposers),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "messages" => Ok(GeneratedField::Messages),
                            "exec" => Ok(GeneratedField::Exec),
                            "title" => Ok(GeneratedField::Title),
                            "summary" => Ok(GeneratedField::Summary),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgSubmitProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgSubmitProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgSubmitProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_policy_address__ = None;
                let mut proposers__ = None;
                let mut metadata__ = None;
                let mut messages__ = None;
                let mut exec__ = None;
                let mut title__ = None;
                let mut summary__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupPolicyAddress => {
                            if group_policy_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyAddress"));
                            }
                            group_policy_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proposers => {
                            if proposers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposers"));
                            }
                            proposers__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::Messages => {
                            if messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messages"));
                            }
                            messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::Exec => {
                            if exec__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exec"));
                            }
                            exec__ = Some(map.next_value::<Exec>()? as i32);
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map.next_value()?);
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgSubmitProposal {
                    group_policy_address: group_policy_address__.unwrap_or_default(),
                    proposers: proposers__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    messages: messages__.unwrap_or_default(),
                    exec: exec__.unwrap_or_default(),
                    title: title__.unwrap_or_default(),
                    summary: summary__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgSubmitProposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgSubmitProposalResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgSubmitProposalResponse", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgSubmitProposalResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgSubmitProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgSubmitProposalResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgSubmitProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(MsgSubmitProposalResponse {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgSubmitProposalResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupAdmin {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if self.group_id != 0 {
            len += 1;
        }
        if !self.new_admin.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupAdmin", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if !self.new_admin.is_empty() {
            struct_ser.serialize_field("newAdmin", &self.new_admin)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupAdmin {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "group_id",
            "groupId",
            "new_admin",
            "newAdmin",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            GroupId,
            NewAdmin,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "newAdmin" | "new_admin" => Ok(GeneratedField::NewAdmin),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUpdateGroupAdmin;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupAdmin")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupAdmin, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut group_id__ = None;
                let mut new_admin__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::NewAdmin => {
                            if new_admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newAdmin"));
                            }
                            new_admin__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgUpdateGroupAdmin {
                    admin: admin__.unwrap_or_default(),
                    group_id: group_id__.unwrap_or_default(),
                    new_admin: new_admin__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupAdmin", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupAdminResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupAdminResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupAdminResponse {
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
            type Value = MsgUpdateGroupAdminResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupAdminResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupAdminResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUpdateGroupAdminResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupAdminResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupMembers {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if self.group_id != 0 {
            len += 1;
        }
        if !self.member_updates.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupMembers", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if !self.member_updates.is_empty() {
            struct_ser.serialize_field("memberUpdates", &self.member_updates)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupMembers {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "group_id",
            "groupId",
            "member_updates",
            "memberUpdates",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            GroupId,
            MemberUpdates,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "memberUpdates" | "member_updates" => Ok(GeneratedField::MemberUpdates),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUpdateGroupMembers;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupMembers")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupMembers, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut group_id__ = None;
                let mut member_updates__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::MemberUpdates => {
                            if member_updates__.is_some() {
                                return Err(serde::de::Error::duplicate_field("memberUpdates"));
                            }
                            member_updates__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgUpdateGroupMembers {
                    admin: admin__.unwrap_or_default(),
                    group_id: group_id__.unwrap_or_default(),
                    member_updates: member_updates__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupMembers", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupMembersResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupMembersResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupMembersResponse {
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
            type Value = MsgUpdateGroupMembersResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupMembersResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupMembersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUpdateGroupMembersResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupMembersResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupMetadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if self.group_id != 0 {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupMetadata", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupMetadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "group_id",
            "groupId",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            GroupId,
            Metadata,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUpdateGroupMetadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupMetadata")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupMetadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut group_id__ = None;
                let mut metadata__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgUpdateGroupMetadata {
                    admin: admin__.unwrap_or_default(),
                    group_id: group_id__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupMetadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupMetadataResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupMetadataResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupMetadataResponse {
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
            type Value = MsgUpdateGroupMetadataResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupMetadataResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupMetadataResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUpdateGroupMetadataResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupMetadataResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupPolicyAdmin {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if !self.group_policy_address.is_empty() {
            len += 1;
        }
        if !self.new_admin.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyAdmin", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if !self.group_policy_address.is_empty() {
            struct_ser.serialize_field("groupPolicyAddress", &self.group_policy_address)?;
        }
        if !self.new_admin.is_empty() {
            struct_ser.serialize_field("newAdmin", &self.new_admin)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupPolicyAdmin {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "group_policy_address",
            "groupPolicyAddress",
            "new_admin",
            "newAdmin",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            GroupPolicyAddress,
            NewAdmin,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "groupPolicyAddress" | "group_policy_address" => Ok(GeneratedField::GroupPolicyAddress),
                            "newAdmin" | "new_admin" => Ok(GeneratedField::NewAdmin),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUpdateGroupPolicyAdmin;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupPolicyAdmin")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupPolicyAdmin, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut group_policy_address__ = None;
                let mut new_admin__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupPolicyAddress => {
                            if group_policy_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyAddress"));
                            }
                            group_policy_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::NewAdmin => {
                            if new_admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("newAdmin"));
                            }
                            new_admin__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgUpdateGroupPolicyAdmin {
                    admin: admin__.unwrap_or_default(),
                    group_policy_address: group_policy_address__.unwrap_or_default(),
                    new_admin: new_admin__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyAdmin", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupPolicyAdminResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyAdminResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupPolicyAdminResponse {
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
            type Value = MsgUpdateGroupPolicyAdminResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupPolicyAdminResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupPolicyAdminResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUpdateGroupPolicyAdminResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyAdminResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupPolicyDecisionPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if !self.group_policy_address.is_empty() {
            len += 1;
        }
        if self.decision_policy.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyDecisionPolicy", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if !self.group_policy_address.is_empty() {
            struct_ser.serialize_field("groupPolicyAddress", &self.group_policy_address)?;
        }
        if let Some(v) = self.decision_policy.as_ref() {
            struct_ser.serialize_field("decisionPolicy", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupPolicyDecisionPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "group_policy_address",
            "groupPolicyAddress",
            "decision_policy",
            "decisionPolicy",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            GroupPolicyAddress,
            DecisionPolicy,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "groupPolicyAddress" | "group_policy_address" => Ok(GeneratedField::GroupPolicyAddress),
                            "decisionPolicy" | "decision_policy" => Ok(GeneratedField::DecisionPolicy),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUpdateGroupPolicyDecisionPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupPolicyDecisionPolicy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupPolicyDecisionPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut group_policy_address__ = None;
                let mut decision_policy__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupPolicyAddress => {
                            if group_policy_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyAddress"));
                            }
                            group_policy_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::DecisionPolicy => {
                            if decision_policy__.is_some() {
                                return Err(serde::de::Error::duplicate_field("decisionPolicy"));
                            }
                            decision_policy__ = map.next_value()?;
                        }
                    }
                }
                Ok(MsgUpdateGroupPolicyDecisionPolicy {
                    admin: admin__.unwrap_or_default(),
                    group_policy_address: group_policy_address__.unwrap_or_default(),
                    decision_policy: decision_policy__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyDecisionPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupPolicyDecisionPolicyResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyDecisionPolicyResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupPolicyDecisionPolicyResponse {
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
            type Value = MsgUpdateGroupPolicyDecisionPolicyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupPolicyDecisionPolicyResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupPolicyDecisionPolicyResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUpdateGroupPolicyDecisionPolicyResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyDecisionPolicyResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupPolicyMetadata {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if !self.group_policy_address.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyMetadata", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if !self.group_policy_address.is_empty() {
            struct_ser.serialize_field("groupPolicyAddress", &self.group_policy_address)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupPolicyMetadata {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "group_policy_address",
            "groupPolicyAddress",
            "metadata",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            GroupPolicyAddress,
            Metadata,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "groupPolicyAddress" | "group_policy_address" => Ok(GeneratedField::GroupPolicyAddress),
                            "metadata" => Ok(GeneratedField::Metadata),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgUpdateGroupPolicyMetadata;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupPolicyMetadata")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupPolicyMetadata, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut group_policy_address__ = None;
                let mut metadata__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::GroupPolicyAddress => {
                            if group_policy_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyAddress"));
                            }
                            group_policy_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgUpdateGroupPolicyMetadata {
                    admin: admin__.unwrap_or_default(),
                    group_policy_address: group_policy_address__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyMetadata", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgUpdateGroupPolicyMetadataResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyMetadataResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgUpdateGroupPolicyMetadataResponse {
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
            type Value = MsgUpdateGroupPolicyMetadataResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgUpdateGroupPolicyMetadataResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgUpdateGroupPolicyMetadataResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgUpdateGroupPolicyMetadataResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgUpdateGroupPolicyMetadataResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgVote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        if self.option != 0 {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.exec != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgVote", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if self.option != 0 {
            let v = VoteOption::from_i32(self.option)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.option)))?;
            struct_ser.serialize_field("option", &v)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if self.exec != 0 {
            let v = Exec::from_i32(self.exec)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.exec)))?;
            struct_ser.serialize_field("exec", &v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgVote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "voter",
            "option",
            "metadata",
            "exec",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Option,
            Metadata,
            Exec,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "voter" => Ok(GeneratedField::Voter),
                            "option" => Ok(GeneratedField::Option),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "exec" => Ok(GeneratedField::Exec),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgVote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgVote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut option__ = None;
                let mut metadata__ = None;
                let mut exec__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Option => {
                            if option__.is_some() {
                                return Err(serde::de::Error::duplicate_field("option"));
                            }
                            option__ = Some(map.next_value::<VoteOption>()? as i32);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::Exec => {
                            if exec__.is_some() {
                                return Err(serde::de::Error::duplicate_field("exec"));
                            }
                            exec__ = Some(map.next_value::<Exec>()? as i32);
                        }
                    }
                }
                Ok(MsgVote {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    option: option__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    exec: exec__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgVote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgVoteResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgVoteResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgVoteResponse {
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
            type Value = MsgVoteResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgVoteResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgVoteResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgVoteResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgVoteResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgWithdrawProposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgWithdrawProposal", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgWithdrawProposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Address,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = MsgWithdrawProposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgWithdrawProposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgWithdrawProposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(MsgWithdrawProposal {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgWithdrawProposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for MsgWithdrawProposalResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let len = 0;
        let struct_ser = serializer.serialize_struct("cosmos.group.v1.MsgWithdrawProposalResponse", len)?;
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for MsgWithdrawProposalResponse {
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
            type Value = MsgWithdrawProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.MsgWithdrawProposalResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<MsgWithdrawProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                while map.next_key::<GeneratedField>()?.is_some() {
                    let _ = map.next_value::<serde::de::IgnoredAny>()?;
                }
                Ok(MsgWithdrawProposalResponse {
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.MsgWithdrawProposalResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for PercentageDecisionPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.percentage.is_empty() {
            len += 1;
        }
        if self.windows.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.PercentageDecisionPolicy", len)?;
        if !self.percentage.is_empty() {
            struct_ser.serialize_field("percentage", &self.percentage)?;
        }
        if let Some(v) = self.windows.as_ref() {
            struct_ser.serialize_field("windows", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for PercentageDecisionPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "percentage",
            "windows",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Percentage,
            Windows,
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
                            "percentage" => Ok(GeneratedField::Percentage),
                            "windows" => Ok(GeneratedField::Windows),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = PercentageDecisionPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.PercentageDecisionPolicy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<PercentageDecisionPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut percentage__ = None;
                let mut windows__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Percentage => {
                            if percentage__.is_some() {
                                return Err(serde::de::Error::duplicate_field("percentage"));
                            }
                            percentage__ = Some(map.next_value()?);
                        }
                        GeneratedField::Windows => {
                            if windows__.is_some() {
                                return Err(serde::de::Error::duplicate_field("windows"));
                            }
                            windows__ = map.next_value()?;
                        }
                    }
                }
                Ok(PercentageDecisionPolicy {
                    percentage: percentage__.unwrap_or_default(),
                    windows: windows__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.PercentageDecisionPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Proposal {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.id != 0 {
            len += 1;
        }
        if !self.group_policy_address.is_empty() {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if !self.proposers.is_empty() {
            len += 1;
        }
        if self.submit_time.is_some() {
            len += 1;
        }
        if self.group_version != 0 {
            len += 1;
        }
        if self.group_policy_version != 0 {
            len += 1;
        }
        if self.status != 0 {
            len += 1;
        }
        if self.final_tally_result.is_some() {
            len += 1;
        }
        if self.voting_period_end.is_some() {
            len += 1;
        }
        if self.executor_result != 0 {
            len += 1;
        }
        if !self.messages.is_empty() {
            len += 1;
        }
        if !self.title.is_empty() {
            len += 1;
        }
        if !self.summary.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.Proposal", len)?;
        if self.id != 0 {
            struct_ser.serialize_field("id", ToString::to_string(&self.id).as_str())?;
        }
        if !self.group_policy_address.is_empty() {
            struct_ser.serialize_field("groupPolicyAddress", &self.group_policy_address)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if !self.proposers.is_empty() {
            struct_ser.serialize_field("proposers", &self.proposers)?;
        }
        if let Some(v) = self.submit_time.as_ref() {
            struct_ser.serialize_field("submitTime", v)?;
        }
        if self.group_version != 0 {
            struct_ser.serialize_field("groupVersion", ToString::to_string(&self.group_version).as_str())?;
        }
        if self.group_policy_version != 0 {
            struct_ser.serialize_field("groupPolicyVersion", ToString::to_string(&self.group_policy_version).as_str())?;
        }
        if self.status != 0 {
            let v = ProposalStatus::from_i32(self.status)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.status)))?;
            struct_ser.serialize_field("status", &v)?;
        }
        if let Some(v) = self.final_tally_result.as_ref() {
            struct_ser.serialize_field("finalTallyResult", v)?;
        }
        if let Some(v) = self.voting_period_end.as_ref() {
            struct_ser.serialize_field("votingPeriodEnd", v)?;
        }
        if self.executor_result != 0 {
            let v = ProposalExecutorResult::from_i32(self.executor_result)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.executor_result)))?;
            struct_ser.serialize_field("executorResult", &v)?;
        }
        if !self.messages.is_empty() {
            struct_ser.serialize_field("messages", &self.messages)?;
        }
        if !self.title.is_empty() {
            struct_ser.serialize_field("title", &self.title)?;
        }
        if !self.summary.is_empty() {
            struct_ser.serialize_field("summary", &self.summary)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Proposal {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "id",
            "group_policy_address",
            "groupPolicyAddress",
            "metadata",
            "proposers",
            "submit_time",
            "submitTime",
            "group_version",
            "groupVersion",
            "group_policy_version",
            "groupPolicyVersion",
            "status",
            "final_tally_result",
            "finalTallyResult",
            "voting_period_end",
            "votingPeriodEnd",
            "executor_result",
            "executorResult",
            "messages",
            "title",
            "summary",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Id,
            GroupPolicyAddress,
            Metadata,
            Proposers,
            SubmitTime,
            GroupVersion,
            GroupPolicyVersion,
            Status,
            FinalTallyResult,
            VotingPeriodEnd,
            ExecutorResult,
            Messages,
            Title,
            Summary,
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
                            "groupPolicyAddress" | "group_policy_address" => Ok(GeneratedField::GroupPolicyAddress),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "proposers" => Ok(GeneratedField::Proposers),
                            "submitTime" | "submit_time" => Ok(GeneratedField::SubmitTime),
                            "groupVersion" | "group_version" => Ok(GeneratedField::GroupVersion),
                            "groupPolicyVersion" | "group_policy_version" => Ok(GeneratedField::GroupPolicyVersion),
                            "status" => Ok(GeneratedField::Status),
                            "finalTallyResult" | "final_tally_result" => Ok(GeneratedField::FinalTallyResult),
                            "votingPeriodEnd" | "voting_period_end" => Ok(GeneratedField::VotingPeriodEnd),
                            "executorResult" | "executor_result" => Ok(GeneratedField::ExecutorResult),
                            "messages" => Ok(GeneratedField::Messages),
                            "title" => Ok(GeneratedField::Title),
                            "summary" => Ok(GeneratedField::Summary),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Proposal;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.Proposal")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Proposal, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut id__ = None;
                let mut group_policy_address__ = None;
                let mut metadata__ = None;
                let mut proposers__ = None;
                let mut submit_time__ = None;
                let mut group_version__ = None;
                let mut group_policy_version__ = None;
                let mut status__ = None;
                let mut final_tally_result__ = None;
                let mut voting_period_end__ = None;
                let mut executor_result__ = None;
                let mut messages__ = None;
                let mut title__ = None;
                let mut summary__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Id => {
                            if id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GroupPolicyAddress => {
                            if group_policy_address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyAddress"));
                            }
                            group_policy_address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::Proposers => {
                            if proposers__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposers"));
                            }
                            proposers__ = Some(map.next_value()?);
                        }
                        GeneratedField::SubmitTime => {
                            if submit_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("submitTime"));
                            }
                            submit_time__ = map.next_value()?;
                        }
                        GeneratedField::GroupVersion => {
                            if group_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupVersion"));
                            }
                            group_version__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::GroupPolicyVersion => {
                            if group_policy_version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicyVersion"));
                            }
                            group_policy_version__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Status => {
                            if status__.is_some() {
                                return Err(serde::de::Error::duplicate_field("status"));
                            }
                            status__ = Some(map.next_value::<ProposalStatus>()? as i32);
                        }
                        GeneratedField::FinalTallyResult => {
                            if final_tally_result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("finalTallyResult"));
                            }
                            final_tally_result__ = map.next_value()?;
                        }
                        GeneratedField::VotingPeriodEnd => {
                            if voting_period_end__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votingPeriodEnd"));
                            }
                            voting_period_end__ = map.next_value()?;
                        }
                        GeneratedField::ExecutorResult => {
                            if executor_result__.is_some() {
                                return Err(serde::de::Error::duplicate_field("executorResult"));
                            }
                            executor_result__ = Some(map.next_value::<ProposalExecutorResult>()? as i32);
                        }
                        GeneratedField::Messages => {
                            if messages__.is_some() {
                                return Err(serde::de::Error::duplicate_field("messages"));
                            }
                            messages__ = Some(map.next_value()?);
                        }
                        GeneratedField::Title => {
                            if title__.is_some() {
                                return Err(serde::de::Error::duplicate_field("title"));
                            }
                            title__ = Some(map.next_value()?);
                        }
                        GeneratedField::Summary => {
                            if summary__.is_some() {
                                return Err(serde::de::Error::duplicate_field("summary"));
                            }
                            summary__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(Proposal {
                    id: id__.unwrap_or_default(),
                    group_policy_address: group_policy_address__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    proposers: proposers__.unwrap_or_default(),
                    submit_time: submit_time__,
                    group_version: group_version__.unwrap_or_default(),
                    group_policy_version: group_policy_version__.unwrap_or_default(),
                    status: status__.unwrap_or_default(),
                    final_tally_result: final_tally_result__,
                    voting_period_end: voting_period_end__,
                    executor_result: executor_result__.unwrap_or_default(),
                    messages: messages__.unwrap_or_default(),
                    title: title__.unwrap_or_default(),
                    summary: summary__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.Proposal", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalExecutorResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PROPOSAL_EXECUTOR_RESULT_UNSPECIFIED",
            Self::NotRun => "PROPOSAL_EXECUTOR_RESULT_NOT_RUN",
            Self::Success => "PROPOSAL_EXECUTOR_RESULT_SUCCESS",
            Self::Failure => "PROPOSAL_EXECUTOR_RESULT_FAILURE",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ProposalExecutorResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PROPOSAL_EXECUTOR_RESULT_UNSPECIFIED",
            "PROPOSAL_EXECUTOR_RESULT_NOT_RUN",
            "PROPOSAL_EXECUTOR_RESULT_SUCCESS",
            "PROPOSAL_EXECUTOR_RESULT_FAILURE",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalExecutorResult;

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
                    .and_then(ProposalExecutorResult::from_i32)
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
                    .and_then(ProposalExecutorResult::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "PROPOSAL_EXECUTOR_RESULT_UNSPECIFIED" => Ok(ProposalExecutorResult::Unspecified),
                    "PROPOSAL_EXECUTOR_RESULT_NOT_RUN" => Ok(ProposalExecutorResult::NotRun),
                    "PROPOSAL_EXECUTOR_RESULT_SUCCESS" => Ok(ProposalExecutorResult::Success),
                    "PROPOSAL_EXECUTOR_RESULT_FAILURE" => Ok(ProposalExecutorResult::Failure),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for ProposalStatus {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "PROPOSAL_STATUS_UNSPECIFIED",
            Self::Submitted => "PROPOSAL_STATUS_SUBMITTED",
            Self::Accepted => "PROPOSAL_STATUS_ACCEPTED",
            Self::Rejected => "PROPOSAL_STATUS_REJECTED",
            Self::Aborted => "PROPOSAL_STATUS_ABORTED",
            Self::Withdrawn => "PROPOSAL_STATUS_WITHDRAWN",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for ProposalStatus {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "PROPOSAL_STATUS_UNSPECIFIED",
            "PROPOSAL_STATUS_SUBMITTED",
            "PROPOSAL_STATUS_ACCEPTED",
            "PROPOSAL_STATUS_REJECTED",
            "PROPOSAL_STATUS_ABORTED",
            "PROPOSAL_STATUS_WITHDRAWN",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ProposalStatus;

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
                    .and_then(ProposalStatus::from_i32)
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
                    .and_then(ProposalStatus::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "PROPOSAL_STATUS_UNSPECIFIED" => Ok(ProposalStatus::Unspecified),
                    "PROPOSAL_STATUS_SUBMITTED" => Ok(ProposalStatus::Submitted),
                    "PROPOSAL_STATUS_ACCEPTED" => Ok(ProposalStatus::Accepted),
                    "PROPOSAL_STATUS_REJECTED" => Ok(ProposalStatus::Rejected),
                    "PROPOSAL_STATUS_ABORTED" => Ok(ProposalStatus::Aborted),
                    "PROPOSAL_STATUS_WITHDRAWN" => Ok(ProposalStatus::Withdrawn),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupInfoRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupInfoRequest", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupInfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupInfoRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryGroupInfoRequest {
                    group_id: group_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupInfoResponse", len)?;
        if let Some(v) = self.info.as_ref() {
            struct_ser.serialize_field("info", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "info",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Info,
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
                            "info" => Ok(GeneratedField::Info),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupInfoResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut info__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Info => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("info"));
                            }
                            info__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupInfoResponse {
                    info: info__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupMembersRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupMembersRequest", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupMembersRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
            Pagination,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupMembersRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupMembersRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupMembersRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupMembersRequest {
                    group_id: group_id__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupMembersRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupMembersResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.members.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupMembersResponse", len)?;
        if !self.members.is_empty() {
            struct_ser.serialize_field("members", &self.members)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupMembersResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "members",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Members,
            Pagination,
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
                            "members" => Ok(GeneratedField::Members),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupMembersResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupMembersResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupMembersResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut members__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Members => {
                            if members__.is_some() {
                                return Err(serde::de::Error::duplicate_field("members"));
                            }
                            members__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupMembersResponse {
                    members: members__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupMembersResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupPoliciesByAdminRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupPoliciesByAdminRequest", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupPoliciesByAdminRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            Pagination,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupPoliciesByAdminRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupPoliciesByAdminRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupPoliciesByAdminRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupPoliciesByAdminRequest {
                    admin: admin__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupPoliciesByAdminRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupPoliciesByAdminResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.group_policies.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupPoliciesByAdminResponse", len)?;
        if !self.group_policies.is_empty() {
            struct_ser.serialize_field("groupPolicies", &self.group_policies)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupPoliciesByAdminResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_policies",
            "groupPolicies",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupPolicies,
            Pagination,
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
                            "groupPolicies" | "group_policies" => Ok(GeneratedField::GroupPolicies),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupPoliciesByAdminResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupPoliciesByAdminResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupPoliciesByAdminResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_policies__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupPolicies => {
                            if group_policies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicies"));
                            }
                            group_policies__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupPoliciesByAdminResponse {
                    group_policies: group_policies__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupPoliciesByAdminResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupPoliciesByGroupRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.group_id != 0 {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupPoliciesByGroupRequest", len)?;
        if self.group_id != 0 {
            struct_ser.serialize_field("groupId", ToString::to_string(&self.group_id).as_str())?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupPoliciesByGroupRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_id",
            "groupId",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupId,
            Pagination,
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
                            "groupId" | "group_id" => Ok(GeneratedField::GroupId),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupPoliciesByGroupRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupPoliciesByGroupRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupPoliciesByGroupRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_id__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupId => {
                            if group_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupId"));
                            }
                            group_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupPoliciesByGroupRequest {
                    group_id: group_id__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupPoliciesByGroupRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupPoliciesByGroupResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.group_policies.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupPoliciesByGroupResponse", len)?;
        if !self.group_policies.is_empty() {
            struct_ser.serialize_field("groupPolicies", &self.group_policies)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupPoliciesByGroupResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "group_policies",
            "groupPolicies",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            GroupPolicies,
            Pagination,
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
                            "groupPolicies" | "group_policies" => Ok(GeneratedField::GroupPolicies),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupPoliciesByGroupResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupPoliciesByGroupResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupPoliciesByGroupResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut group_policies__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::GroupPolicies => {
                            if group_policies__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groupPolicies"));
                            }
                            group_policies__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupPoliciesByGroupResponse {
                    group_policies: group_policies__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupPoliciesByGroupResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupPolicyInfoRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupPolicyInfoRequest", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupPolicyInfoRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
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
                            "address" => Ok(GeneratedField::Address),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupPolicyInfoRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupPolicyInfoRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupPolicyInfoRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryGroupPolicyInfoRequest {
                    address: address__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupPolicyInfoRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupPolicyInfoResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.info.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupPolicyInfoResponse", len)?;
        if let Some(v) = self.info.as_ref() {
            struct_ser.serialize_field("info", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupPolicyInfoResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "info",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Info,
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
                            "info" => Ok(GeneratedField::Info),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupPolicyInfoResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupPolicyInfoResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupPolicyInfoResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut info__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Info => {
                            if info__.is_some() {
                                return Err(serde::de::Error::duplicate_field("info"));
                            }
                            info__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupPolicyInfoResponse {
                    info: info__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupPolicyInfoResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupsByAdminRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.admin.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupsByAdminRequest", len)?;
        if !self.admin.is_empty() {
            struct_ser.serialize_field("admin", &self.admin)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupsByAdminRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "admin",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Admin,
            Pagination,
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
                            "admin" => Ok(GeneratedField::Admin),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupsByAdminRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupsByAdminRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupsByAdminRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut admin__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Admin => {
                            if admin__.is_some() {
                                return Err(serde::de::Error::duplicate_field("admin"));
                            }
                            admin__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupsByAdminRequest {
                    admin: admin__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupsByAdminRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupsByAdminResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.groups.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupsByAdminResponse", len)?;
        if !self.groups.is_empty() {
            struct_ser.serialize_field("groups", &self.groups)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupsByAdminResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "groups",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Groups,
            Pagination,
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
                            "groups" => Ok(GeneratedField::Groups),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupsByAdminResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupsByAdminResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupsByAdminResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut groups__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Groups => {
                            if groups__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groups"));
                            }
                            groups__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupsByAdminResponse {
                    groups: groups__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupsByAdminResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupsByMemberRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupsByMemberRequest", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupsByMemberRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Pagination,
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
                            "address" => Ok(GeneratedField::Address),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupsByMemberRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupsByMemberRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupsByMemberRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupsByMemberRequest {
                    address: address__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupsByMemberRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryGroupsByMemberResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.groups.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryGroupsByMemberResponse", len)?;
        if !self.groups.is_empty() {
            struct_ser.serialize_field("groups", &self.groups)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryGroupsByMemberResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "groups",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Groups,
            Pagination,
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
                            "groups" => Ok(GeneratedField::Groups),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryGroupsByMemberResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryGroupsByMemberResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryGroupsByMemberResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut groups__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Groups => {
                            if groups__.is_some() {
                                return Err(serde::de::Error::duplicate_field("groups"));
                            }
                            groups__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryGroupsByMemberResponse {
                    groups: groups__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryGroupsByMemberResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryProposalRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryProposalRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryProposalRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryProposalRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryProposalRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalResponse {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryProposalResponse", len)?;
        if let Some(v) = self.proposal.as_ref() {
            struct_ser.serialize_field("proposal", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposal,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryProposalResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposal => {
                            if proposal__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposal"));
                            }
                            proposal__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryProposalResponse {
                    proposal: proposal__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryProposalResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalsByGroupPolicyRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.address.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryProposalsByGroupPolicyRequest", len)?;
        if !self.address.is_empty() {
            struct_ser.serialize_field("address", &self.address)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalsByGroupPolicyRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "address",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Address,
            Pagination,
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
                            "address" => Ok(GeneratedField::Address),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryProposalsByGroupPolicyRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryProposalsByGroupPolicyRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalsByGroupPolicyRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut address__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Address => {
                            if address__.is_some() {
                                return Err(serde::de::Error::duplicate_field("address"));
                            }
                            address__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryProposalsByGroupPolicyRequest {
                    address: address__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryProposalsByGroupPolicyRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryProposalsByGroupPolicyResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.proposals.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryProposalsByGroupPolicyResponse", len)?;
        if !self.proposals.is_empty() {
            struct_ser.serialize_field("proposals", &self.proposals)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryProposalsByGroupPolicyResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposals",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Proposals,
            Pagination,
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
                            "proposals" => Ok(GeneratedField::Proposals),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryProposalsByGroupPolicyResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryProposalsByGroupPolicyResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryProposalsByGroupPolicyResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposals__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Proposals => {
                            if proposals__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposals"));
                            }
                            proposals__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryProposalsByGroupPolicyResponse {
                    proposals: proposals__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryProposalsByGroupPolicyResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTallyResultRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryTallyResultRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTallyResultRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTallyResultRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryTallyResultRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTallyResultRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                    }
                }
                Ok(QueryTallyResultRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryTallyResultRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryTallyResultResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.tally.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryTallyResultResponse", len)?;
        if let Some(v) = self.tally.as_ref() {
            struct_ser.serialize_field("tally", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryTallyResultResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "tally",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Tally,
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
                            "tally" => Ok(GeneratedField::Tally),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryTallyResultResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryTallyResultResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryTallyResultResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut tally__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Tally => {
                            if tally__.is_some() {
                                return Err(serde::de::Error::duplicate_field("tally"));
                            }
                            tally__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryTallyResultResponse {
                    tally: tally__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryTallyResultResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVoteByProposalVoterRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryVoteByProposalVoterRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVoteByProposalVoterRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "voter",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "voter" => Ok(GeneratedField::Voter),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVoteByProposalVoterRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryVoteByProposalVoterRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVoteByProposalVoterRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(QueryVoteByProposalVoterRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryVoteByProposalVoterRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVoteByProposalVoterResponse {
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
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryVoteByProposalVoterResponse", len)?;
        if let Some(v) = self.vote.as_ref() {
            struct_ser.serialize_field("vote", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVoteByProposalVoterResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "vote",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Vote,
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
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVoteByProposalVoterResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryVoteByProposalVoterResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVoteByProposalVoterResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut vote__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Vote => {
                            if vote__.is_some() {
                                return Err(serde::de::Error::duplicate_field("vote"));
                            }
                            vote__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryVoteByProposalVoterResponse {
                    vote: vote__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryVoteByProposalVoterResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVotesByProposalRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryVotesByProposalRequest", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVotesByProposalRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Pagination,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVotesByProposalRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryVotesByProposalRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVotesByProposalRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryVotesByProposalRequest {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryVotesByProposalRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVotesByProposalResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.votes.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryVotesByProposalResponse", len)?;
        if !self.votes.is_empty() {
            struct_ser.serialize_field("votes", &self.votes)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVotesByProposalResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "votes",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Votes,
            Pagination,
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
                            "votes" => Ok(GeneratedField::Votes),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVotesByProposalResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryVotesByProposalResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVotesByProposalResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut votes__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Votes => {
                            if votes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votes"));
                            }
                            votes__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryVotesByProposalResponse {
                    votes: votes__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryVotesByProposalResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVotesByVoterRequest {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.voter.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryVotesByVoterRequest", len)?;
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVotesByVoterRequest {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "voter",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Voter,
            Pagination,
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
                            "voter" => Ok(GeneratedField::Voter),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVotesByVoterRequest;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryVotesByVoterRequest")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVotesByVoterRequest, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut voter__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryVotesByVoterRequest {
                    voter: voter__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryVotesByVoterRequest", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for QueryVotesByVoterResponse {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.votes.is_empty() {
            len += 1;
        }
        if self.pagination.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.QueryVotesByVoterResponse", len)?;
        if !self.votes.is_empty() {
            struct_ser.serialize_field("votes", &self.votes)?;
        }
        if let Some(v) = self.pagination.as_ref() {
            struct_ser.serialize_field("pagination", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for QueryVotesByVoterResponse {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "votes",
            "pagination",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Votes,
            Pagination,
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
                            "votes" => Ok(GeneratedField::Votes),
                            "pagination" => Ok(GeneratedField::Pagination),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = QueryVotesByVoterResponse;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.QueryVotesByVoterResponse")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<QueryVotesByVoterResponse, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut votes__ = None;
                let mut pagination__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Votes => {
                            if votes__.is_some() {
                                return Err(serde::de::Error::duplicate_field("votes"));
                            }
                            votes__ = Some(map.next_value()?);
                        }
                        GeneratedField::Pagination => {
                            if pagination__.is_some() {
                                return Err(serde::de::Error::duplicate_field("pagination"));
                            }
                            pagination__ = map.next_value()?;
                        }
                    }
                }
                Ok(QueryVotesByVoterResponse {
                    votes: votes__.unwrap_or_default(),
                    pagination: pagination__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.QueryVotesByVoterResponse", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for TallyResult {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.yes_count.is_empty() {
            len += 1;
        }
        if !self.abstain_count.is_empty() {
            len += 1;
        }
        if !self.no_count.is_empty() {
            len += 1;
        }
        if !self.no_with_veto_count.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.TallyResult", len)?;
        if !self.yes_count.is_empty() {
            struct_ser.serialize_field("yesCount", &self.yes_count)?;
        }
        if !self.abstain_count.is_empty() {
            struct_ser.serialize_field("abstainCount", &self.abstain_count)?;
        }
        if !self.no_count.is_empty() {
            struct_ser.serialize_field("noCount", &self.no_count)?;
        }
        if !self.no_with_veto_count.is_empty() {
            struct_ser.serialize_field("noWithVetoCount", &self.no_with_veto_count)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for TallyResult {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "yes_count",
            "yesCount",
            "abstain_count",
            "abstainCount",
            "no_count",
            "noCount",
            "no_with_veto_count",
            "noWithVetoCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            YesCount,
            AbstainCount,
            NoCount,
            NoWithVetoCount,
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
                            "yesCount" | "yes_count" => Ok(GeneratedField::YesCount),
                            "abstainCount" | "abstain_count" => Ok(GeneratedField::AbstainCount),
                            "noCount" | "no_count" => Ok(GeneratedField::NoCount),
                            "noWithVetoCount" | "no_with_veto_count" => Ok(GeneratedField::NoWithVetoCount),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = TallyResult;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.TallyResult")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<TallyResult, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut yes_count__ = None;
                let mut abstain_count__ = None;
                let mut no_count__ = None;
                let mut no_with_veto_count__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::YesCount => {
                            if yes_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("yesCount"));
                            }
                            yes_count__ = Some(map.next_value()?);
                        }
                        GeneratedField::AbstainCount => {
                            if abstain_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("abstainCount"));
                            }
                            abstain_count__ = Some(map.next_value()?);
                        }
                        GeneratedField::NoCount => {
                            if no_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noCount"));
                            }
                            no_count__ = Some(map.next_value()?);
                        }
                        GeneratedField::NoWithVetoCount => {
                            if no_with_veto_count__.is_some() {
                                return Err(serde::de::Error::duplicate_field("noWithVetoCount"));
                            }
                            no_with_veto_count__ = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TallyResult {
                    yes_count: yes_count__.unwrap_or_default(),
                    abstain_count: abstain_count__.unwrap_or_default(),
                    no_count: no_count__.unwrap_or_default(),
                    no_with_veto_count: no_with_veto_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.TallyResult", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ThresholdDecisionPolicy {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.threshold.is_empty() {
            len += 1;
        }
        if self.windows.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.ThresholdDecisionPolicy", len)?;
        if !self.threshold.is_empty() {
            struct_ser.serialize_field("threshold", &self.threshold)?;
        }
        if let Some(v) = self.windows.as_ref() {
            struct_ser.serialize_field("windows", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ThresholdDecisionPolicy {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "threshold",
            "windows",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Threshold,
            Windows,
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
                            "threshold" => Ok(GeneratedField::Threshold),
                            "windows" => Ok(GeneratedField::Windows),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ThresholdDecisionPolicy;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.ThresholdDecisionPolicy")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<ThresholdDecisionPolicy, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut threshold__ = None;
                let mut windows__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::Threshold => {
                            if threshold__.is_some() {
                                return Err(serde::de::Error::duplicate_field("threshold"));
                            }
                            threshold__ = Some(map.next_value()?);
                        }
                        GeneratedField::Windows => {
                            if windows__.is_some() {
                                return Err(serde::de::Error::duplicate_field("windows"));
                            }
                            windows__ = map.next_value()?;
                        }
                    }
                }
                Ok(ThresholdDecisionPolicy {
                    threshold: threshold__.unwrap_or_default(),
                    windows: windows__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.ThresholdDecisionPolicy", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for Vote {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.proposal_id != 0 {
            len += 1;
        }
        if !self.voter.is_empty() {
            len += 1;
        }
        if self.option != 0 {
            len += 1;
        }
        if !self.metadata.is_empty() {
            len += 1;
        }
        if self.submit_time.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("cosmos.group.v1.Vote", len)?;
        if self.proposal_id != 0 {
            struct_ser.serialize_field("proposalId", ToString::to_string(&self.proposal_id).as_str())?;
        }
        if !self.voter.is_empty() {
            struct_ser.serialize_field("voter", &self.voter)?;
        }
        if self.option != 0 {
            let v = VoteOption::from_i32(self.option)
                .ok_or_else(|| serde::ser::Error::custom(format!("Invalid variant {}", self.option)))?;
            struct_ser.serialize_field("option", &v)?;
        }
        if !self.metadata.is_empty() {
            struct_ser.serialize_field("metadata", &self.metadata)?;
        }
        if let Some(v) = self.submit_time.as_ref() {
            struct_ser.serialize_field("submitTime", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for Vote {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "proposal_id",
            "proposalId",
            "voter",
            "option",
            "metadata",
            "submit_time",
            "submitTime",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            ProposalId,
            Voter,
            Option,
            Metadata,
            SubmitTime,
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
                            "proposalId" | "proposal_id" => Ok(GeneratedField::ProposalId),
                            "voter" => Ok(GeneratedField::Voter),
                            "option" => Ok(GeneratedField::Option),
                            "metadata" => Ok(GeneratedField::Metadata),
                            "submitTime" | "submit_time" => Ok(GeneratedField::SubmitTime),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = Vote;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct cosmos.group.v1.Vote")
            }

            fn visit_map<V>(self, mut map: V) -> std::result::Result<Vote, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut proposal_id__ = None;
                let mut voter__ = None;
                let mut option__ = None;
                let mut metadata__ = None;
                let mut submit_time__ = None;
                while let Some(k) = map.next_key()? {
                    match k {
                        GeneratedField::ProposalId => {
                            if proposal_id__.is_some() {
                                return Err(serde::de::Error::duplicate_field("proposalId"));
                            }
                            proposal_id__ = 
                                Some(map.next_value::<::pbjson::private::NumberDeserialize<_>>()?.0)
                            ;
                        }
                        GeneratedField::Voter => {
                            if voter__.is_some() {
                                return Err(serde::de::Error::duplicate_field("voter"));
                            }
                            voter__ = Some(map.next_value()?);
                        }
                        GeneratedField::Option => {
                            if option__.is_some() {
                                return Err(serde::de::Error::duplicate_field("option"));
                            }
                            option__ = Some(map.next_value::<VoteOption>()? as i32);
                        }
                        GeneratedField::Metadata => {
                            if metadata__.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata__ = Some(map.next_value()?);
                        }
                        GeneratedField::SubmitTime => {
                            if submit_time__.is_some() {
                                return Err(serde::de::Error::duplicate_field("submitTime"));
                            }
                            submit_time__ = map.next_value()?;
                        }
                    }
                }
                Ok(Vote {
                    proposal_id: proposal_id__.unwrap_or_default(),
                    voter: voter__.unwrap_or_default(),
                    option: option__.unwrap_or_default(),
                    metadata: metadata__.unwrap_or_default(),
                    submit_time: submit_time__,
                })
            }
        }
        deserializer.deserialize_struct("cosmos.group.v1.Vote", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for VoteOption {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let variant = match self {
            Self::Unspecified => "VOTE_OPTION_UNSPECIFIED",
            Self::Yes => "VOTE_OPTION_YES",
            Self::Abstain => "VOTE_OPTION_ABSTAIN",
            Self::No => "VOTE_OPTION_NO",
            Self::NoWithVeto => "VOTE_OPTION_NO_WITH_VETO",
        };
        serializer.serialize_str(variant)
    }
}
impl<'de> serde::Deserialize<'de> for VoteOption {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "VOTE_OPTION_UNSPECIFIED",
            "VOTE_OPTION_YES",
            "VOTE_OPTION_ABSTAIN",
            "VOTE_OPTION_NO",
            "VOTE_OPTION_NO_WITH_VETO",
        ];

        struct GeneratedVisitor;

        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = VoteOption;

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
                    .and_then(VoteOption::from_i32)
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
                    .and_then(VoteOption::from_i32)
                    .ok_or_else(|| {
                        serde::de::Error::invalid_value(serde::de::Unexpected::Unsigned(v), &self)
                    })
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "VOTE_OPTION_UNSPECIFIED" => Ok(VoteOption::Unspecified),
                    "VOTE_OPTION_YES" => Ok(VoteOption::Yes),
                    "VOTE_OPTION_ABSTAIN" => Ok(VoteOption::Abstain),
                    "VOTE_OPTION_NO" => Ok(VoteOption::No),
                    "VOTE_OPTION_NO_WITH_VETO" => Ok(VoteOption::NoWithVeto),
                    _ => Err(serde::de::Error::unknown_variant(value, FIELDS)),
                }
            }
        }
        deserializer.deserialize_any(GeneratedVisitor)
    }
}
