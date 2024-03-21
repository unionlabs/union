// https://github.com/rust-lang/rust/issues/35853#issuecomment-415993963
macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

macro_rules! try_from_relayer_msg {
    (
        chain = $Chain:ty,
        generics = ($($generics:tt)+),
        msgs = $Enum:ident(
            $($Variant:ident($Ty:ty),)+
        ),
    ) => {
        with_dollar_sign! {
            ($d:tt) => {
                macro_rules! with_generics {
                    (
                        chain = $d Chain:ty,
                        msgs = $d Enum:ident(
                            $d ($d Variant:ident($d Ty:ty),)+
                        ),
                    ) => {
                        $d (
                            impl <$($generics)+> TryFrom<queue_msg::QueueMsg<crate::RelayerMsgTypes>> for crate::Identified<$d Chain, Tr, $d Ty>
                            where
                                crate::Identified<$d Chain, Tr, crate::data::Data<$d Chain, Tr>>: TryFrom<crate::AnyLightClientIdentified<crate::data::AnyData>, Error = crate::AnyLightClientIdentified<crate::data::AnyData>> + Into<crate::AnyLightClientIdentified<crate::data::AnyData>>
                            {
                                type Error = queue_msg::QueueMsg<crate::RelayerMsgTypes>;
                                fn try_from(value: queue_msg::QueueMsg<crate::RelayerMsgTypes>) -> Result<crate::Identified<$d Chain, Tr, $d Ty>, queue_msg::QueueMsg<crate::RelayerMsgTypes>> {
                                    match value {
                                        queue_msg::QueueMsg::Data(data) => {
                                            let crate::Identified::<$d Chain, Tr, crate::data::Data<$d Chain, Tr>> {
                                                chain_id,
                                                t,
                                                __marker: _,
                                            } = data.try_into().map_err(queue_msg::QueueMsg::Data)?;

                                            match t {
                                                crate::data::Data::LightClientSpecific(
                                                    crate::data::LightClientSpecificData($d Enum::$d Variant(
                                                    t,
                                                ))) => Ok(crate::id::<$d Chain, Tr, $d Ty>(chain_id, t)),
                                                _ => Err(queue_msg::QueueMsg::Data(Into::<crate::AnyLightClientIdentified<crate::data::AnyData>>::into(crate::id(chain_id, t))))
                                            }

                                        },
                                        _ => Err(value),
                                    }
                                }
                            }

                            impl <$($generics)+> From<crate::Identified<$d Chain, Tr, $d Ty>> for crate::AnyLightClientIdentified<crate::data::AnyData>
                            where
                                crate::AnyLightClientIdentified<crate::data::AnyData>: From<crate::Identified<$d Chain, Tr, crate::data::Data<$d Chain, Tr>>>
                            {
                                fn from(crate::Identified { chain_id, t, __marker: _ }: crate::Identified<$d Chain, Tr, $d Ty>) -> crate::AnyLightClientIdentified<crate::data::AnyData> {
                                    crate::AnyLightClientIdentified::from(crate::id(
                                        chain_id,
                                        crate::data::Data::LightClientSpecific(crate::data::LightClientSpecificData($d Enum::$d Variant(
                                            t,
                                        ))),
                                    ))
                                }
                            }

                            impl <$($generics)+> TryFrom<crate::AnyLightClientIdentified<crate::data::AnyData>> for crate::Identified<$d Chain, Tr, $d Ty>
                            where
                                crate::Identified<$d Chain, Tr, crate::data::Data<$d Chain, Tr>>: TryFrom<crate::AnyLightClientIdentified<crate::data::AnyData>, Error = crate::AnyLightClientIdentified<crate::data::AnyData>> + Into<crate::AnyLightClientIdentified<crate::data::AnyData>>
                            {
                                type Error = crate::AnyLightClientIdentified<crate::data::AnyData>;

                                fn try_from(value: crate::AnyLightClientIdentified<crate::data::AnyData>) -> Result<crate::Identified<$d Chain, Tr, $d Ty>, crate::AnyLightClientIdentified<crate::data::AnyData>> {
                                    let crate::Identified {
                                        chain_id,
                                        t,
                                        __marker: _,
                                    } = value.try_into()?;

                                    match t {
                                        crate::data::Data::LightClientSpecific(crate::data::LightClientSpecificData($d Enum::$d Variant(
                                            t,
                                        ))) => Ok(crate::id(chain_id, t)),
                                        _ => Err(Into::<crate::AnyLightClientIdentified<crate::data::AnyData>>::into(crate::id(chain_id, t)))
                                    }
                                }
                            }
                        )+
                    };
                }
            }
        }

        with_generics!(
            chain = $Chain,
            msgs = $Enum(
                $($Variant($Ty),)+
            ),
        );
    };
}

// functionality common between all cosmos-sdk chains
pub mod cosmos_sdk;

pub mod cosmos;
pub mod union;

pub mod ethereum;
