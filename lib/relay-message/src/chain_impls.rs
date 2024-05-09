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
        const _: () = {
            use crate::{AnyLightClientIdentified, RelayMessageTypes, id, identified, Identified, data::{AnyData, Data, LightClientSpecificData}};
            use queue_msg::QueueMsg;

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
                                impl <$($generics)+> TryFrom<QueueMsg<RelayMessageTypes>> for Identified<$d Chain, Tr, $d Ty>
                                where
                                    identified!(Data<$d Chain, Tr>): TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>> + Into<AnyLightClientIdentified<AnyData>>
                                {
                                    type Error = QueueMsg<RelayMessageTypes>;
                                    fn try_from(value: QueueMsg<RelayMessageTypes>) -> Result<Identified<$d Chain, Tr, $d Ty>, QueueMsg<RelayMessageTypes>> {
                                        match value {
                                            QueueMsg::Data(data) => {
                                                let Identified {
                                                    chain_id,
                                                    t,
                                                    __marker: _,
                                                } = data.try_into().map_err(QueueMsg::Data)?;

                                                match t {
                                                    Data::LightClientSpecific(
                                                        LightClientSpecificData($d Enum::$d Variant(
                                                        t,
                                                    ))) => Ok(id(chain_id, t)),
                                                    _ => Err(QueueMsg::Data(Into::<AnyLightClientIdentified<AnyData>>::into(crate::id(chain_id, t))))
                                                }

                                            },
                                            _ => Err(value),
                                        }
                                    }
                                }

                                impl <$($generics)+> From<Identified<$d Chain, Tr, $d Ty>> for AnyLightClientIdentified<AnyData>
                                where
                                    AnyLightClientIdentified<AnyData>: From<identified!(Data<$d Chain, Tr>)>
                                {
                                    fn from(Identified { chain_id, t, __marker: _ }: Identified<$d Chain, Tr, $d Ty>) -> AnyLightClientIdentified<AnyData> {
                                        AnyLightClientIdentified::from(id(
                                            chain_id,
                                            Data::LightClientSpecific(LightClientSpecificData($d Enum::$d Variant(
                                                t,
                                            ))),
                                        ))
                                    }
                                }

                                impl <$($generics)+> TryFrom<AnyLightClientIdentified<AnyData>> for Identified<$d Chain, Tr, $d Ty>
                                where
                                    identified!(Data<$d Chain, Tr>): TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>> + Into<AnyLightClientIdentified<AnyData>>
                                {
                                    type Error = AnyLightClientIdentified<AnyData>;

                                    fn try_from(value: AnyLightClientIdentified<AnyData>) -> Result<Identified<$d Chain, Tr, $d Ty>, AnyLightClientIdentified<AnyData>> {
                                        let Identified {
                                            chain_id,
                                            t,
                                            __marker: _,
                                        } = value.try_into()?;

                                        match t {
                                            Data::LightClientSpecific(LightClientSpecificData($d Enum::$d Variant(
                                                t,
                                            ))) => Ok(id(chain_id, t)),
                                            _ => Err(Into::<AnyLightClientIdentified<AnyData>>::into(id(chain_id, t)))
                                        }
                                    }
                                }
                            )+

                            // impl From<<$d Chain as LightClient>::$d LcMsg> for $d Specific<$d Chain> {
                            //     fn from(msg: <$d Chain as LightClient>::$d LcMsg) -> Self {
                            //         Self(msg)
                            //     }
                            // }
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
    };
}

// functionality common between all cosmos-sdk chains
pub mod cosmos_sdk;

pub mod cosmos;
pub mod union;

pub mod arbitrum;
pub mod ethereum;
pub mod scroll;
