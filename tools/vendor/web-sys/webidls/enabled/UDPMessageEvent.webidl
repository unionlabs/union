/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/raw-sockets/#interface-udpmessageevent
 */

//Bug 1056444: This interface should be removed after UDPSocket.input/UDPSocket.output are ready.
[Constructor(DOMString type, optional UDPMessageEventInit eventInitDict),
 Pref="dom.udpsocket.enabled",
 ChromeOnly]
interface UDPMessageEvent : Event {
    readonly    attribute DOMString      remoteAddress;
    readonly    attribute unsigned short remotePort;
    readonly    attribute any            data;
};

dictionary UDPMessageEventInit : EventInit {
  DOMString remoteAddress = "";
  unsigned short remotePort = 0;
  any data = null;
};
