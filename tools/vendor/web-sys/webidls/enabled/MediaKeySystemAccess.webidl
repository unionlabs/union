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

enum MediaKeysRequirement {
  "required",
  "optional",
  "not-allowed"
};

dictionary MediaKeySystemMediaCapability {
   DOMString contentType = "";
   DOMString robustness = "";
};

dictionary MediaKeySystemConfiguration {
  DOMString                               label = "";
  sequence<DOMString>                     initDataTypes = [];
  sequence<MediaKeySystemMediaCapability> audioCapabilities = [];
  sequence<MediaKeySystemMediaCapability> videoCapabilities = [];
  MediaKeysRequirement                    distinctiveIdentifier = "optional";
  MediaKeysRequirement                    persistentState = "optional";
  sequence<DOMString>                     sessionTypes;
};

interface MediaKeySystemAccess {
  readonly    attribute DOMString keySystem;
  [NewObject]
  MediaKeySystemConfiguration getConfiguration();
  [NewObject]
  Promise<MediaKeys> createMediaKeys();
};
