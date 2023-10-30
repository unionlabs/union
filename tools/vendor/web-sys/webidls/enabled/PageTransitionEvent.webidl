/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/**
 * The PageTransitionEvent interface is used for the pageshow and
 * pagehide events, which are generic events that apply to both page
 * load/unload and saving/restoring a document from session history.
 */

[Constructor(DOMString type, optional PageTransitionEventInit eventInitDict)]
interface PageTransitionEvent : Event
{
  /**
   * Set to true if the document has been or will be persisted across
   * firing of the event.  For example, if a document is being cached in
   * session history, |persisted| is true for the PageHide event.
   */
  readonly attribute boolean persisted;

  // Whether the document is in the middle of a frame swap.
  [ChromeOnly]
  readonly attribute boolean inFrameSwap;
};

dictionary PageTransitionEventInit : EventInit
{
  boolean persisted = false;
  boolean inFrameSwap = false;
};
