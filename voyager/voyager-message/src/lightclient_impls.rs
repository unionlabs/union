macro_rules! try_from_relayer_msg {
    (
        #[
            $Lc:ident($(
                lc_msg(
                    msg = $LcMsg:ident($Specific:ident),
                    ty = $Msg:ident,
                    variants($( $Var:ident($Ty:ty), )+),
                ),
            )+)
        ]
    ) => {
        $(
            $(
                impl TryFrom<RelayerMsg> for Identified<$Lc, $Ty> {
                    type Error = RelayerMsg;
                    fn try_from(value: RelayerMsg) -> Result<Identified<$Lc, $Ty>, RelayerMsg> {
                        match value {
                            RelayerMsg::$LcMsg(crate::AnyLightClientIdentified::$Lc(Identified {
                                chain_id,
                                data:
                                    $LcMsg::LightClientSpecific($Specific($Msg::$Var(
                                        data,
                                    ))),
                            })) => Ok(Identified { chain_id, data }),
                            _ => Err(value),
                        }
                    }
                }
            )+

            crate::lightclient_impls::this_is_a_hack_look_away! {
                $Lc(
                    lc_msg(
                        msg = $LcMsg($Specific),
                        ty = $Msg,
                        variants($( $Var($Ty), )+),
                    ),
                )
            }

            impl From<<$Lc as LightClient>::$LcMsg> for $Specific<$Lc> {
                fn from(msg: <$Lc as LightClient>::$LcMsg) -> Self {
                    Self(msg)
                }
            }
        )+
    };
}

macro_rules! this_is_a_hack_look_away {
    (
            $Lc:ident(
                lc_msg(
                    msg = Data(LightClientSpecificData),
                    ty = $Msg:ident,
                    variants($( $Var:ident($Ty:ty), )+),
                ),
            )
    ) => {
        $(
            impl From<Identified<$Lc, $Ty>> for crate::AggregateData {
                fn from(Identified { chain_id, data }: Identified<$Lc, $Ty>) -> crate::AggregateData {
                    crate::AggregateData::$Lc(Identified {
                        chain_id,
                        data: Data::LightClientSpecific(LightClientSpecificData($Msg::$Var(
                            data,
                        ))),
                    })
                }
            }

            impl TryFrom<crate::AggregateData> for Identified<$Lc, $Ty> {
                type Error = crate::AggregateData;

                fn try_from(value: crate::AggregateData) -> Result<Identified<$Lc, $Ty>, crate::AggregateData> {
                    match value {
                        crate::AnyLightClientIdentified::$Lc(Identified {
                            chain_id,
                            data: Data::LightClientSpecific(LightClientSpecificData($Msg::$Var(
                                data,
                            ))),
                        }) => Ok(Identified { chain_id, data }),
                        _ => Err(value),
                    }
                }
            }
        )+
    };

    ($($_:tt)*) => {};
}

use this_is_a_hack_look_away;

pub mod cometbls;
pub mod ethereum;
