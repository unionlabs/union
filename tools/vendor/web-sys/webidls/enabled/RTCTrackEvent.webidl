/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://w3c.github.io/webrtc-pc/#idl-def-RTCTrackEvent
 */

dictionary RTCTrackEventInit : EventInit {
    required RTCRtpReceiver        receiver;
    required MediaStreamTrack      track;
    sequence<MediaStream> streams = [];
    required RTCRtpTransceiver     transceiver;
};

[Pref="media.peerconnection.enabled",
 Constructor(DOMString type, RTCTrackEventInit eventInitDict)]
interface RTCTrackEvent : Event {
    readonly        attribute RTCRtpReceiver           receiver;
    readonly        attribute MediaStreamTrack         track;

// TODO: Use FrozenArray once available. (Bug 1236777)
//  readonly        attribute FrozenArray<MediaStream> streams;

    [Frozen, Cached, Pure]
    readonly        attribute sequence<MediaStream> streams; // workaround
    readonly        attribute RTCRtpTransceiver transceiver;
};
