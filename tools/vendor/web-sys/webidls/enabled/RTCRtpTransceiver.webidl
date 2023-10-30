/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://w3c.github.io/webrtc-pc/#rtcrtptransceiver-interface
 */

enum RTCRtpTransceiverDirection {
    "sendrecv",
    "sendonly",
    "recvonly",
    "inactive"
};

dictionary RTCRtpTransceiverInit {
    RTCRtpTransceiverDirection         direction = "sendrecv";
    sequence<MediaStream>              streams = [];
    // TODO: bug 1396918
    // sequence<RTCRtpEncodingParameters> sendEncodings;
};

[Pref="media.peerconnection.enabled",
 JSImplementation="@mozilla.org/dom/rtptransceiver;1"]
interface RTCRtpTransceiver {
    readonly attribute DOMString?                  mid;
    [SameObject]
    readonly attribute RTCRtpSender                sender;
    [SameObject]
    readonly attribute RTCRtpReceiver              receiver;
    readonly attribute boolean                     stopped;
             attribute RTCRtpTransceiverDirection  direction;
    readonly attribute RTCRtpTransceiverDirection? currentDirection;

    undefined stop();
    // TODO: bug 1396922
    // undefined setCodecPreferences(sequence<RTCRtpCodecCapability> codecs);

    [ChromeOnly]
    undefined setRemoteTrackId(DOMString trackId);
    [ChromeOnly]
    boolean remoteTrackIdIs(DOMString trackId);

    // Mostly for testing
    [Pref="media.peerconnection.remoteTrackId.enabled"]
    DOMString getRemoteTrackId();

    [ChromeOnly]
    undefined setAddTrackMagic();
    [ChromeOnly]
    readonly attribute boolean addTrackMagic;
    [ChromeOnly]
    attribute boolean shouldRemove;
    [ChromeOnly]
    undefined setCurrentDirection(RTCRtpTransceiverDirection direction);
    [ChromeOnly]
    undefined setDirectionInternal(RTCRtpTransceiverDirection direction);
    [ChromeOnly]
    undefined setMid(DOMString mid);
    [ChromeOnly]
    undefined unsetMid();
    [ChromeOnly]
    undefined setStopped();

    [ChromeOnly]
    DOMString getKind();
    [ChromeOnly]
    boolean hasBeenUsedToSend();
    [ChromeOnly]
    undefined sync();

    [ChromeOnly]
    undefined insertDTMF(DOMString tones,
                    optional unsigned long duration = 100,
                    optional unsigned long interToneGap = 70);
};

