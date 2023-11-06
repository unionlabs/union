use crate::macros::wrapper_enum;

wrapper_enum! {
    #[proto(protos::cosmos::ics23::v1::HashOp)]
    pub enum HashOp {
        NoHash = 0,
        Sha256 = 1,
        Sha512 = 2,
        Keccak = 3,
        Ripemd160 = 4,
        Bitcoin = 5,
        Sha512256 = 6,
    }
}
