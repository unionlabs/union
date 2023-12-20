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
                            impl <$($generics)+> TryFrom<RelayerMsg> for Identified<$d Chain, Tr, $d Ty>
                            where
                                identified!(Data<$d Chain, Tr>): TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>> + Into<AnyLightClientIdentified<AnyData>>
                            {
                                type Error = RelayerMsg;
                                fn try_from(value: RelayerMsg) -> Result<Identified<$d Chain, Tr, $d Ty>, RelayerMsg> {
                                    match value {
                                        RelayerMsg::Data(data) => {
                                            let Identified {
                                                chain_id,
                                                data,
                                                __marker: _,
                                            } = data.try_into().map_err(RelayerMsg::Data)?;

                                            match data {
                                                crate::Data::LightClientSpecific(LightClientSpecificData($d Enum::$d Variant(
                                                    data,
                                                ))) => Ok(Identified::new(chain_id, data)),
                                                _ => Err(RelayerMsg::Data(Into::<AnyLightClientIdentified<AnyData>>::into(Identified::new(chain_id, data))))
                                            }

                                        },
                                        _ => Err(value),
                                    }
                                }
                            }

                            impl <$($generics)+> From<Identified<$d Chain, Tr, $d Ty>> for crate::AnyLightClientIdentified<crate::data::AnyData>
                            where
                                AnyLightClientIdentified<AnyData>: From<identified!(Data<$d Chain, Tr>)>
                            {
                                fn from(Identified { chain_id, data, __marker: _ }: Identified<$d Chain, Tr, $d Ty>) -> crate::AnyLightClientIdentified<crate::data::AnyData> {
                                    crate::AnyLightClientIdentified::from(Identified::new(
                                        chain_id,
                                        Data::LightClientSpecific(LightClientSpecificData($d Enum::$d Variant(
                                            data,
                                        ))),
                                    ))
                                }
                            }

                            impl <$($generics)+> TryFrom<crate::AnyLightClientIdentified<crate::data::AnyData>> for Identified<$d Chain, Tr, $d Ty>
                            where
                                identified!(Data<$d Chain, Tr>): TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>> + Into<AnyLightClientIdentified<AnyData>>
                            {
                                type Error = crate::AnyLightClientIdentified<crate::data::AnyData>;

                                fn try_from(value: crate::AnyLightClientIdentified<crate::data::AnyData>) -> Result<Identified<$d Chain, Tr, $d Ty>, crate::AnyLightClientIdentified<crate::data::AnyData>> {
                                    let Identified {
                                        chain_id,
                                        data,
                                        __marker: _,
                                    } = value.try_into()?;

                                    match data {
                                        Data::LightClientSpecific(LightClientSpecificData($d Enum::$d Variant(
                                            data,
                                        ))) => Ok(Identified::new(chain_id, data)),
                                        _ => Err(Into::<AnyLightClientIdentified<AnyData>>::into(Identified::new(chain_id, data)))
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
}

pub mod cosmos;
pub mod evm;
pub mod union;
