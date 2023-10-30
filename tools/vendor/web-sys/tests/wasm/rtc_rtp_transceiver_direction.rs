use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

use web_sys::{
    RtcPeerConnection, RtcRtpTransceiver, RtcRtpTransceiverDirection, RtcRtpTransceiverInit,
    RtcSessionDescriptionInit,
};

#[wasm_bindgen(
    inline_js = "export function is_unified_avail() { return Object.keys(RTCRtpTransceiver.prototype).indexOf('currentDirection')>-1; }"
)]
extern "C" {
    /// Available in FF since forever, in Chrome since 72, in Safari since 12.1
    fn is_unified_avail() -> bool;
}

#[wasm_bindgen_test]
async fn rtc_rtp_transceiver_direction() {
    if !is_unified_avail() {
        return;
    }

    let mut tr_init: RtcRtpTransceiverInit = RtcRtpTransceiverInit::new();

    let pc1: RtcPeerConnection = RtcPeerConnection::new().unwrap();

    let tr1: RtcRtpTransceiver = pc1.add_transceiver_with_str_and_init(
        "audio",
        tr_init.direction(RtcRtpTransceiverDirection::Sendonly),
    );
    assert_eq!(tr1.direction(), RtcRtpTransceiverDirection::Sendonly);
    assert_eq!(tr1.current_direction(), None);

    let pc2: RtcPeerConnection = RtcPeerConnection::new().unwrap();

    let (_, p2) = exchange_sdps(pc1, pc2).await;
    assert_eq!(tr1.direction(), RtcRtpTransceiverDirection::Sendonly);
    assert_eq!(
        tr1.current_direction(),
        Some(RtcRtpTransceiverDirection::Sendonly)
    );

    let tr2: RtcRtpTransceiver = js_sys::try_iter(&p2.get_transceivers())
        .unwrap()
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .unchecked_into();

    assert_eq!(tr2.direction(), RtcRtpTransceiverDirection::Recvonly);
    assert_eq!(
        tr2.current_direction(),
        Some(RtcRtpTransceiverDirection::Recvonly)
    );
}

async fn exchange_sdps(
    p1: RtcPeerConnection,
    p2: RtcPeerConnection,
) -> (RtcPeerConnection, RtcPeerConnection) {
    let offer = JsFuture::from(p1.create_offer()).await.unwrap();
    let offer = offer.unchecked_into::<RtcSessionDescriptionInit>();
    JsFuture::from(p1.set_local_description(&offer))
        .await
        .unwrap();
    JsFuture::from(p2.set_remote_description(&offer))
        .await
        .unwrap();
    let answer = JsFuture::from(p2.create_answer()).await.unwrap();
    let answer = answer.unchecked_into::<RtcSessionDescriptionInit>();
    JsFuture::from(p2.set_local_description(&answer))
        .await
        .unwrap();
    JsFuture::from(p1.set_remote_description(&answer))
        .await
        .unwrap();
    (p1, p2)
}
