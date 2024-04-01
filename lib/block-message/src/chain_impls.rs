// https://github.com/rust-lang/rust/issues/35853#issuecomment-415993963
macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

macro_rules! try_from_block_poll_msg {
    (
        chain = $Chain:ty,
        generics = ($($generics:tt)*),
        msgs = $Enum:ident(
            $($Variant:ident($Ty:ty),)+
        ),
        $(where = ($($where:tt)+))?
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
                            impl <$($generics)*> TryFrom<QueueMsg<crate::BlockMessageTypes>> for Identified<$d Chain, $d Ty>
                            where
                                Identified<$d Chain, Data<$d Chain>>: TryFrom<AnyChainIdentified<AnyData>, Error = AnyChainIdentified<AnyData>> + Into<AnyChainIdentified<AnyData>>,
                                $($($where)+)?
                            {
                                type Error = QueueMsg<crate::BlockMessageTypes>;
                                fn try_from(value: QueueMsg<crate::BlockMessageTypes>) -> Result<Identified<$d Chain, $d Ty>, QueueMsg<BlockMessageTypes>> {
                                    match value {
                                        QueueMsg::Data(data) => {
                                            let Identified {
                                                chain_id,
                                                t,
                                            } = data.try_into().map_err(QueueMsg::Data)?;

                                            match t {
                                                crate::data::Data::ChainSpecific(
                                                    crate::data::ChainSpecificData($d Enum::$d Variant(
                                                    t,
                                                ))) => Ok(Identified::new(chain_id, t)),
                                                _ => Err(QueueMsg::Data(Into::<AnyChainIdentified<AnyData>>::into(Identified::<$d Chain, _>::new(chain_id, t))))
                                            }

                                        },
                                        _ => Err(value),
                                    }
                                }
                            }

                            impl <$($generics)*> From<Identified<$d Chain, $d Ty>> for crate::AnyChainIdentified<crate::data::AnyData>
                            where
                                AnyChainIdentified<AnyData>: From<Identified<$d Chain, Data<$d Chain>>>,
                                $($($where)+)?
                           {
                                fn from(Identified { chain_id, t, }: Identified<$d Chain, $d Ty>) -> crate::AnyChainIdentified<crate::data::AnyData> {
                                    crate::AnyChainIdentified::<crate::data::AnyData>::from(Identified::<$d Chain, _>::new(
                                        chain_id,
                                        Data::ChainSpecific(crate::data::ChainSpecificData($d Enum::$d Variant(
                                            t,
                                        ))),
                                    ))
                                }
                            }

                            impl <$($generics)*> TryFrom<crate::AnyChainIdentified<crate::data::AnyData>> for Identified<$d Chain, $d Ty>
                            where
                                Identified<$d Chain, Data<$d Chain>>: TryFrom<AnyChainIdentified<AnyData>, Error = AnyChainIdentified<AnyData>> + Into<AnyChainIdentified<AnyData>>,
                                $($($where)+)?
                            {
                                type Error = crate::AnyChainIdentified<crate::data::AnyData>;

                                fn try_from(value: crate::AnyChainIdentified<crate::data::AnyData>) -> Result<Identified<$d Chain, $d Ty>, crate::AnyChainIdentified<crate::data::AnyData>> {
                                    let Identified::<$d Chain, _> {
                                        chain_id,
                                        t,
                                    } = value.try_into()?;

                                    match t {
                                        Data::ChainSpecific(crate::data::ChainSpecificData($d Enum::$d Variant(
                                            t,
                                        ))) => Ok(Identified::new(chain_id, t)),
                                        _ => Err(Into::<AnyChainIdentified<AnyData>>::into(Identified::new(chain_id, t)))
                                    }
                                }
                            }
                        )+

                        // impl From<<$d Chain as Chain>::$d LcMsg> for $d Specific<$d Chain> {
                        //     fn from(msg: <$d Chain as Chain>::$d LcMsg) -> Self {
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
pub mod ethereum;
pub mod scroll;
pub mod union;

pub mod cosmos_sdk;
