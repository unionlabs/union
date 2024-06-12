use macros::apply;

use crate::macros::wrapper_enum;

#[apply(wrapper_enum)]
#[model(proto(protos::cosmos::tx::signing::v1beta1::SignMode))]
pub enum SignMode {
    Unspecified = 0,
    Direct = 1,
    Textual = 2,
    DirectAux = 3,
    LegacyAminoJson = 127,
    Eip191 = 191,
}
