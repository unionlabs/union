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

interface MediaKeySession : EventTarget {
  // error state
  readonly attribute MediaKeyError? error;

  // session properties
  readonly attribute DOMString sessionId;

  readonly attribute unrestricted double expiration;

  readonly attribute Promise<undefined> closed;

  readonly attribute MediaKeyStatusMap keyStatuses;

  attribute EventHandler onkeystatuseschange;

  attribute EventHandler onmessage;

  [NewObject]
  Promise<undefined> generateRequest(DOMString initDataType, BufferSource initData);

  [NewObject]
  Promise<boolean> load(DOMString sessionId);

  // session operations
  [NewObject]
  Promise<undefined> update(BufferSource response);

  [NewObject]
  Promise<undefined> close();

  [NewObject]
  Promise<undefined> remove();
};
