/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/2011/webrtc/editor/webrtc.html#idl-def-RTCPeerConnectionIceEvent
 */

dictionary RTCPeerConnectionIceEventInit : EventInit {
  RTCIceCandidate? candidate = null;
};

[Pref="media.peerconnection.enabled",
 Constructor(DOMString type,
             optional RTCPeerConnectionIceEventInit eventInitDict)]
interface RTCPeerConnectionIceEvent : Event {
  readonly attribute RTCIceCandidate? candidate;
};
