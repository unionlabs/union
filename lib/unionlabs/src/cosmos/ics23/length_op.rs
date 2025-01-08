use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::cosmos::ics23::v1::LengthOp))]
    pub enum LengthOp {
        NoPrefix = 0,
        VarProto = 1,
        VarRlp = 2,
        Fixed32Big = 3,
        Fixed32Little = 4,
        Fixed64Big = 5,
        Fixed64Little = 6,
        Require32Bytes = 7,
        Require64Bytes = 8,
    }
}
