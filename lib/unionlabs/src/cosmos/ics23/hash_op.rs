use crate::macros::wrapper_enum;

wrapper_enum! {
    #[model(proto(protos::cosmos::ics23::v1::HashOp))]
    #[derive(Default)]
    pub enum HashOp {
        #[default]
        NoHash = 0,
        Sha256 = 1,
        Sha512 = 2,
        Keccak256 = 3,
        Ripemd160 = 4,
        Bitcoin = 5,
        Sha512256 = 6,
        Blake2b512 = 7,
        Blake2s256 = 8,
        Blake3 = 9,
        MiMC = 10,
    }
}
