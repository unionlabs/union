/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[Constructor(DOMString typeArg, optional KeyboardEventInit keyboardEventInitDict)]
interface KeyboardEvent : UIEvent
{
  readonly attribute unsigned long    charCode;
  [NeedsCallerType]
  readonly attribute unsigned long    keyCode;

  [NeedsCallerType]
  readonly attribute boolean          altKey;
  [NeedsCallerType]
  readonly attribute boolean          ctrlKey;
  [NeedsCallerType]
  readonly attribute boolean          shiftKey;
  readonly attribute boolean          metaKey;

  [NeedsCallerType]
  boolean getModifierState(DOMString key);

  const unsigned long DOM_KEY_LOCATION_STANDARD = 0x00;
  const unsigned long DOM_KEY_LOCATION_LEFT     = 0x01;
  const unsigned long DOM_KEY_LOCATION_RIGHT    = 0x02;
  const unsigned long DOM_KEY_LOCATION_NUMPAD   = 0x03;

  readonly attribute unsigned long location;
  readonly attribute boolean       repeat;
  readonly attribute boolean       isComposing;

  readonly attribute DOMString key;
  [NeedsCallerType]
  readonly attribute DOMString code;

  [Throws]
  undefined initKeyboardEvent(DOMString typeArg,
                         optional boolean bubblesArg = false,
                         optional boolean cancelableArg = false,
                         optional Window? viewArg = null,
                         optional DOMString keyArg = "",
                         optional unsigned long locationArg = 0,
                         optional boolean ctrlKey = false,
                         optional boolean altKey = false,
                         optional boolean shiftKey = false,
                         optional boolean metaKey = false);

  // This returns the initialized dictionary for generating a
  // same-type keyboard event
  [Cached, ChromeOnly, Constant]
  readonly attribute KeyboardEventInit initDict;
};

dictionary KeyboardEventInit : EventModifierInit
{
  DOMString      key           = "";
  DOMString      code          = "";
  unsigned long  location      = 0;
  boolean        repeat        = false;
  boolean        isComposing   = false;

  // legacy attributes
  unsigned long  charCode      = 0;
  unsigned long  keyCode       = 0;
  unsigned long  which         = 0;
};
