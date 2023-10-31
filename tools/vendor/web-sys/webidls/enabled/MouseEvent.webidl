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

[Constructor(DOMString typeArg, optional MouseEventInit mouseEventInitDict)]
interface MouseEvent : UIEvent {
  [NeedsCallerType]
  readonly attribute long           screenX;
  [NeedsCallerType]
  readonly attribute long           screenY;
  readonly attribute long           clientX;
  readonly attribute long           clientY;
  [BinaryName="clientX"]
  readonly attribute long           x;
  [BinaryName="clientY"]
  readonly attribute long           y;
  readonly attribute long           offsetX;
  readonly attribute long           offsetY;
  readonly attribute boolean        ctrlKey;
  readonly attribute boolean        shiftKey;
  readonly attribute boolean        altKey;
  readonly attribute boolean        metaKey;
  readonly attribute short          button;
  readonly attribute unsigned short buttons;
  readonly attribute EventTarget?   relatedTarget;
  readonly attribute DOMString?     region;

  // Pointer Lock
  readonly attribute long           movementX;
  readonly attribute long           movementY;

  // Deprecated in DOM Level 3:
undefined initMouseEvent(DOMString typeArg,
                    optional boolean canBubbleArg = false,
                    optional boolean cancelableArg = false,
                    optional Window? viewArg = null,
                    optional long detailArg = 0,
                    optional long screenXArg = 0,
                    optional long screenYArg = 0,
                    optional long clientXArg = 0,
                    optional long clientYArg = 0,
                    optional boolean ctrlKeyArg = false,
                    optional boolean altKeyArg = false,
                    optional boolean shiftKeyArg = false,
                    optional boolean metaKeyArg = false,
                    optional short buttonArg = 0,
                    optional EventTarget? relatedTargetArg = null);
  // Introduced in DOM Level 3:
  boolean                           getModifierState(DOMString keyArg);
};

// Suggested initMouseEvent replacement initializer:
dictionary MouseEventInit : EventModifierInit {
  // Attributes for MouseEvent:
  long           screenX       = 0;
  long           screenY       = 0;
  long           clientX       = 0;
  long           clientY       = 0;
  short          button        = 0;
  // Note: "buttons" was not previously initializable through initMouseEvent!
  unsigned short buttons       = 0;
  EventTarget?   relatedTarget = null;

  // Pointer Lock
  long           movementX = 0;
  long           movementY = 0;
};
