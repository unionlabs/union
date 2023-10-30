/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/* Dispatched as part of the "error" event in the following situations:
* - if there's an error detected when the TCPSocket closes
* - if there's an internal error while sending data
* - if there's an error connecting to the host
*/

[Constructor(DOMString type, optional TCPSocketErrorEventInit eventInitDict),
 Func="mozilla::dom::TCPSocket::ShouldTCPSocketExist",
 Exposed=(Window,System)]
interface TCPSocketErrorEvent : Event {
  readonly attribute DOMString name;
  readonly attribute DOMString message;
};

dictionary TCPSocketErrorEventInit : EventInit
{
  DOMString name = "";
  DOMString message = "";
};
