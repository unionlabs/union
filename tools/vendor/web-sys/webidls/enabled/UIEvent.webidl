/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface please see
 * http://dev.w3.org/2006/webapi/DOM-Level-3-Events/html/DOM3-Events.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional UIEventInit eventInitDict)]
interface UIEvent : Event
{
  readonly attribute WindowProxy? view;
  readonly attribute long         detail;
  undefined initUIEvent(DOMString aType,
                   optional boolean aCanBubble = false,
                   optional boolean aCancelable = false,
                   optional Window? aView = null,
                   optional long aDetail = 0);
};

// Additional DOM0 properties.
partial interface UIEvent {
  const long SCROLL_PAGE_UP = -32768;
  const long SCROLL_PAGE_DOWN = 32768;

  readonly attribute long          layerX;
  readonly attribute long          layerY;
  readonly attribute long          pageX;
  readonly attribute long          pageY;
  [NeedsCallerType]
  readonly attribute unsigned long which;
  readonly attribute Node?         rangeParent;
  readonly attribute long          rangeOffset;
};

dictionary UIEventInit : EventInit
{
  Window? view = null;
  long    detail = 0;
};

// NOTE: Gecko doesn't support commented out modifiers yet.
dictionary EventModifierInit : UIEventInit
{
  boolean ctrlKey = false;
  boolean shiftKey = false;
  boolean altKey = false;
  boolean metaKey = false;
  boolean modifierAltGraph = false;
  boolean modifierCapsLock = false;
  boolean modifierFn = false;
  boolean modifierFnLock = false;
  // boolean modifierHyper = false;
  boolean modifierNumLock = false;
  boolean modifierOS = false;
  boolean modifierScrollLock = false;
  // boolean modifierSuper = false;
  boolean modifierSymbol = false;
  boolean modifierSymbolLock = false;
};
