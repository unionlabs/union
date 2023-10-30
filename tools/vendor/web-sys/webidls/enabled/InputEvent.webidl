/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[Constructor(DOMString type, optional InputEventInit eventInitDict)]
interface InputEvent : UIEvent
{
  readonly attribute boolean       isComposing;
  readonly attribute DOMString     inputType;
  [NeedsCallerType]
  readonly attribute DOMString?    data;
};

dictionary InputEventInit : UIEventInit
{
  boolean isComposing = false;
  DOMString inputType = "";
  // NOTE:  Currently, default value of `data` attribute is declared as empty
  //        string by UI Events.  However, both Chrome and Safari uses `null`,
  //        and there is a spec issue about this:
  //        https://github.com/w3c/uievents/issues/139
  //        So, we take `null` for compatibility with them.
  DOMString? data = null;
};

partial interface InputEvent
{
  [NeedsCallerType]
  readonly attribute DataTransfer? dataTransfer;
  // Enable `getTargetRanges()` only when `beforeinput` event is enabled
  // because this may be used for feature detection of `beforeinput` event
  // support (due to Chrome not supporting `onbeforeinput` attribute).
  [Pref="dom.input_events.beforeinput.enabled"]
  sequence<StaticRange> getTargetRanges();
};

partial dictionary InputEventInit
{
  DataTransfer? dataTransfer = null;
  [Pref="dom.input_events.beforeinput.enabled"]
  sequence<StaticRange> targetRanges = [];
};