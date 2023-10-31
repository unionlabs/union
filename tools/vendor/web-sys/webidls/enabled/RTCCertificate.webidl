/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Specification: http://w3c.github.io/webrtc-pc/#certificate-management
 */

dictionary RTCCertificateExpiration {
  [EnforceRange]
  DOMTimeStamp expires;
};

[Pref="media.peerconnection.enabled"]
interface RTCCertificate {
  readonly attribute DOMTimeStamp expires;
};
