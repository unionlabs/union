/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-a-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-a-element
[HTMLConstructor]
interface HTMLAnchorElement : HTMLElement {
           [CEReactions, SetterThrows]
           attribute DOMString target;
           [CEReactions, SetterThrows]
           attribute DOMString download;
           [CEReactions, SetterThrows]
           attribute DOMString ping;
           [CEReactions, SetterThrows]
           attribute DOMString rel;
           [CEReactions, SetterThrows]
           attribute DOMString referrerPolicy;
           [PutForwards=value]
  readonly attribute DOMTokenList relList;
           [CEReactions, SetterThrows]
           attribute DOMString hreflang;
           [CEReactions, SetterThrows]
           attribute DOMString type;

           [CEReactions, Throws]
           attribute DOMString text;
};

HTMLAnchorElement includes HTMLHyperlinkElementUtils;

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLAnchorElement {
           [CEReactions, SetterThrows]
           attribute DOMString coords;
           [CEReactions, SetterThrows]
           attribute DOMString charset;
           [CEReactions, SetterThrows]
           attribute DOMString name;
           [CEReactions, SetterThrows]
           attribute DOMString rev;
           [CEReactions, SetterThrows]
           attribute DOMString shape;
};
