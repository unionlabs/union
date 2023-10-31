/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/2011/webrtc/editor/webrtc.html#idl-def-RTCConfiguration
 */

enum RTCIceCredentialType {
    "password",
    "token"
};

dictionary RTCIceServer {
    (DOMString or sequence<DOMString>) urls;
    DOMString  url; //deprecated
    DOMString username;
    DOMString credential;
    RTCIceCredentialType credentialType = "password";
};

enum RTCIceTransportPolicy {
    "relay",
    "all"
};

enum RTCBundlePolicy {
    "balanced",
    "max-compat",
    "max-bundle"
};

dictionary RTCConfiguration {
    sequence<RTCIceServer> iceServers;
    RTCIceTransportPolicy  iceTransportPolicy = "all";
    RTCBundlePolicy bundlePolicy = "balanced";
    DOMString? peerIdentity = null;
    sequence<RTCCertificate> certificates;
};
