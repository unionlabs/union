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
interface HTMLTableElement : HTMLElement {
           [CEReactions, SetterThrows]
           attribute HTMLTableCaptionElement? caption;
  HTMLElement createCaption();
  [CEReactions]
  undefined deleteCaption();
           [CEReactions, SetterThrows]
           attribute HTMLTableSectionElement? tHead;
  HTMLElement createTHead();
  [CEReactions]
  undefined deleteTHead();
           [CEReactions, SetterThrows]
           attribute HTMLTableSectionElement? tFoot;
  HTMLElement createTFoot();
  [CEReactions]
  undefined deleteTFoot();
  readonly attribute HTMLCollection tBodies;
  HTMLElement createTBody();
  readonly attribute HTMLCollection rows;
  [Throws]
  HTMLElement insertRow(optional long index = -1);
  [CEReactions, Throws]
  undefined deleteRow(long index);
  //         attribute boolean sortable;
  //undefined stopSorting();
};

partial interface HTMLTableElement {
           [CEReactions, SetterThrows]
           attribute DOMString align;
           [CEReactions, SetterThrows]
           attribute DOMString border;
           [CEReactions, SetterThrows]
           attribute DOMString frame;
           [CEReactions, SetterThrows]
           attribute DOMString rules;
           [CEReactions, SetterThrows]
           attribute DOMString summary;
           [CEReactions, SetterThrows]
           attribute DOMString width;

  [CEReactions, TreatNullAs=EmptyString, SetterThrows]
           attribute DOMString bgColor;
  [CEReactions, TreatNullAs=EmptyString, SetterThrows]
           attribute DOMString cellPadding;
  [CEReactions, TreatNullAs=EmptyString, SetterThrows]
           attribute DOMString cellSpacing;
};
