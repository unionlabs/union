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

enum MediaKeyMessageType {
  "license-request",
  "license-renewal",
  "license-release",
  "individualization-request"
};

[Constructor(DOMString type, MediaKeyMessageEventInit eventInitDict)]
interface MediaKeyMessageEvent : Event {
  readonly attribute MediaKeyMessageType messageType;
  [Throws]
  readonly attribute ArrayBuffer message;
};

dictionary MediaKeyMessageEventInit : EventInit {
  required MediaKeyMessageType messageType;
  required ArrayBuffer message;
};
