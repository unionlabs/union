/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/html-media/raw-file/default/encrypted-media/encrypted-media.html
 *
 * Copyright © 2014 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved.
 * W3C liability, trademark and document use rules apply.
 */

// Note: "persistent-usage-record" session type is unsupported yet, as
// it's marked as "at risk" in the spec, and Chrome doesn't support it. 
enum MediaKeySessionType {
  "temporary",
  "persistent-license",
  // persistent-usage-record,
};

// https://github.com/WICG/media-capabilities/blob/master/eme-extension-policy-check.md
dictionary MediaKeysPolicy {
  DOMString minHdcpVersion = "";
};

interface MediaKeys {
  readonly attribute DOMString keySystem;

  [NewObject, Throws]
  MediaKeySession createSession(optional MediaKeySessionType sessionType = "temporary");

  [NewObject]
  Promise<undefined> setServerCertificate(BufferSource serverCertificate);

  [Pref="media.eme.hdcp-policy-check.enabled", NewObject]
  Promise<MediaKeyStatus> getStatusForPolicy(optional MediaKeysPolicy policy);
};
