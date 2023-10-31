/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

/**
 * TCPSocketEvent is the event dispatched for all of the events described by TCPSocket,
 * except the "error" event. It contains the socket that was associated with the event,
 * the type of event, and the data associated with the event if the event is a "data" event.
 */

[Constructor(DOMString type, optional TCPSocketEventInit eventInitDict),
 Func="mozilla::dom::TCPSocket::ShouldTCPSocketExist",
 Exposed=(Window,System)]
interface TCPSocketEvent : Event {
  /**
   * If the event is a "data" event, data will be the bytes read from the network;
   * if the binaryType of the socket was "arraybuffer", this value will be of type
   * ArrayBuffer, otherwise, it will be a ByteString.
   *
   * For other events, data will be an empty string.
   */
  //TODO: make this (ArrayBuffer or ByteString) after sorting out the rooting required. (bug 1121634)
  readonly attribute any data;
};

dictionary TCPSocketEventInit : EventInit {
  //TODO: make this (ArrayBuffer or ByteString) after sorting out the rooting required. (bug 1121634)
  any data = null;
};
