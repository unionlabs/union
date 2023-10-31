/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

[HTMLConstructor]
interface HTMLTableCellElement : HTMLElement {
           [CEReactions, SetterThrows]
           attribute unsigned long colSpan;
           [CEReactions, SetterThrows]
           attribute unsigned long rowSpan;
  //[PutForwards=value] readonly attribute DOMTokenList headers;
           [CEReactions, SetterThrows]
           attribute DOMString headers;
  readonly attribute long cellIndex;
};

partial interface HTMLTableCellElement {
           [CEReactions, SetterThrows]
           attribute DOMString align;
           [CEReactions, SetterThrows]
           attribute DOMString axis;
           [CEReactions, SetterThrows]
           attribute DOMString height;
           [CEReactions, SetterThrows]
           attribute DOMString width;

           [CEReactions, SetterThrows]
           attribute DOMString ch;
           [CEReactions, SetterThrows]
           attribute DOMString chOff;
           [CEReactions, SetterThrows]
           attribute boolean noWrap;
           [CEReactions, SetterThrows]
           attribute DOMString vAlign;

  [CEReactions, TreatNullAs=EmptyString, SetterThrows]
           attribute DOMString bgColor;
};
