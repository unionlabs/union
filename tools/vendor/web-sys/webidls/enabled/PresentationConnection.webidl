/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/presentation-api/#interface-presentationconnection
 */

enum PresentationConnectionState
{
  // The initial state when a PresentationConnection is ceated.
  "connecting",

  // Existing presentation, and the communication channel is active.
  "connected",

  // Existing presentation, but the communication channel is inactive.
  "closed",

  // The presentation is nonexistent anymore. It could be terminated manually,
  // or either controlling or receiving browsing context is no longer available.
  "terminated"
};

enum PresentationConnectionBinaryType
{
  "blob",
  "arraybuffer"
};

[Pref="dom.presentation.enabled"]
interface PresentationConnection : EventTarget {
  /*
   * Unique id for all existing connections.
   */
  [Constant]
  readonly attribute DOMString id;

  /*
   * Specifies the connection's presentation URL.
   */
  readonly attribute DOMString url;

  /*
   * @value "connected", "closed", or "terminated".
   */
  readonly attribute PresentationConnectionState state;

  attribute EventHandler onconnect;
  attribute EventHandler onclose;
  attribute EventHandler onterminate;
  attribute PresentationConnectionBinaryType binaryType;

  /*
   * After a communication channel has been established between the controlling
   * and receiving context, this function is called to send message out, and the
   * event handler "onmessage" will be invoked at the remote side.
   *
   * This function only works when the state is "connected".
   */
  [Throws]
  undefined send(DOMString data);

  [Throws]
  undefined send(Blob data);

  [Throws]
  undefined send(ArrayBuffer data);

  [Throws]
  undefined send(ArrayBufferView data);

  /*
   * It is triggered when receiving messages.
   */
  attribute EventHandler onmessage;

  /*
   * Both the controlling and receiving browsing context can close the
   * connection. Then the connection state should turn into "closed".
   *
   * This function only works when the state is "connected" or "connecting".
   */
  [Throws]
  undefined close();

  /*
   * Both the controlling and receiving browsing context can terminate the
   * connection. Then the connection state should turn into "terminated".
   *
   * This function only works when the state is not "connected".
   */
   [Throws]
   undefined terminate();
};
