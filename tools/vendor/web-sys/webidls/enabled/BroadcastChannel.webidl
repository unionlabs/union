/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface, please see
 * https://html.spec.whatwg.org/#broadcastchannel
 */

[Constructor(DOMString channel),
 Exposed=(Window,Worker)]
interface BroadcastChannel : EventTarget {
  readonly attribute DOMString name;

  [Throws]
  undefined postMessage(any message);

  undefined close();

  attribute EventHandler onmessage;
  attribute EventHandler onmessageerror;
};
