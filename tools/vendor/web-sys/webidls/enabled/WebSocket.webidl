/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/html/#network
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and Opera Software ASA.
 * You are granted a license to use, reproduce and create derivative works of this document.
 */

enum BinaryType { "blob", "arraybuffer" };

[Exposed=(Window,Worker),
 Constructor(DOMString url),
 Constructor(DOMString url, DOMString protocols),
 Constructor(DOMString url, sequence<DOMString> protocols)]
interface WebSocket : EventTarget {

  readonly attribute DOMString url;

  // ready state
  const unsigned short CONNECTING = 0;
  const unsigned short OPEN = 1;
  const unsigned short CLOSING = 2;
  const unsigned short CLOSED = 3;

  readonly attribute unsigned short readyState;

  readonly attribute unsigned long bufferedAmount;

  // networking

  attribute EventHandler onopen;

  attribute EventHandler onerror;

  attribute EventHandler onclose;

  readonly attribute DOMString extensions;

  readonly attribute DOMString protocol;

  [Throws]
  undefined close([Clamp] optional unsigned short code, optional DOMString reason);

  // messaging

  attribute EventHandler onmessage;

  attribute BinaryType binaryType;

  [Throws]
  undefined send(DOMString data);

  [Throws]
  undefined send(Blob data);

  [Throws]
  undefined send(ArrayBuffer data);

  [Throws]
  undefined send(ArrayBufferView data);
};

// Support for creating server-side chrome-only WebSocket. Used in
// devtools remote debugging server.
// invalid widl
// interface nsITransportProvider;

partial interface WebSocket {
  [ChromeOnly, NewObject, Throws]
  static WebSocket createServerWebSocket(DOMString url,
                                         sequence<DOMString> protocols,
                                         nsITransportProvider transportProvider,
                                         DOMString negotiatedExtensions);
};
