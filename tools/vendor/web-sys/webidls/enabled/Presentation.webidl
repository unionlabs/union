/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/presentation-api/#interface-presentation
 */

[Pref="dom.presentation.enabled"]
interface Presentation {
 /*
  * This should be used by the UA as the default presentation request for the
  * controller. When the UA wishes to initiate a PresentationConnection on the
  * controller's behalf, it MUST start a presentation connection using the default
  * presentation request (as if the controller had called |defaultRequest.start()|).
  *
  * Only used by controlling browsing context (senders).
  */
  [Pref="dom.presentation.controller.enabled"]
  attribute PresentationRequest? defaultRequest;

  /*
   * This should be available on the receiving browsing context in order to
   * access the controlling browsing context and communicate with them.
   */
  [SameObject,
   Pref="dom.presentation.receiver.enabled"]
  readonly attribute PresentationReceiver? receiver;
};
