/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/presentation-api/#interface-presentationconnectioncloseevent
 */

enum PresentationConnectionClosedReason
{
  // The communication encountered an unrecoverable error.
  "error",

  // |PresentationConnection.close()| is called by controlling browsing context
  // or the receiving browsing context.
  "closed",

  // The connection is closed because the destination browsing context
  // that owned the connection navigated or was discarded.
  "wentaway"
};

[Constructor(DOMString type,
             PresentationConnectionCloseEventInit eventInitDict),
 Pref="dom.presentation.enabled"]
interface PresentationConnectionCloseEvent : Event
{
  readonly attribute PresentationConnectionClosedReason reason;

  // The message is a human readable description of
  // how the communication channel encountered an error.
  // It is empty when the closed reason is closed or wentaway.
  readonly attribute DOMString message;
};

dictionary PresentationConnectionCloseEventInit : EventInit
{
  required PresentationConnectionClosedReason reason;
  DOMString message = "";
};
