use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MsgSendResponse {}

#[doc(hidden)] // TODO: Do this to all proto and ethabi modules
pub mod proto {
    use super::MsgSendResponse;
    use crate::impl_proto_via_try_from_into;

    impl_proto_via_try_from_into!(MsgSendResponse => protos::cosmos::bank::v1beta1::MsgSendResponse);

    impl From<MsgSendResponse> for protos::cosmos::bank::v1beta1::MsgSendResponse {
        fn from(_: MsgSendResponse) -> Self {
            Self {}
        }
    }

    impl From<protos::cosmos::bank::v1beta1::MsgSendResponse> for MsgSendResponse {
        fn from(_: protos::cosmos::bank::v1beta1::MsgSendResponse) -> Self {
            Self {}
        }
    }
}
