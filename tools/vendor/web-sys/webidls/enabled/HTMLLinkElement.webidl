/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-link-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-link-element
[HTMLConstructor]
interface HTMLLinkElement : HTMLElement {
  [Pure]
           attribute boolean disabled;
  [CEReactions, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows, Pure]
           attribute DOMString href;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString? crossOrigin;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString rel;
  [PutForwards=value]
  readonly attribute DOMTokenList relList;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString media;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString hreflang;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString type;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString referrerPolicy;
  [PutForwards=value] readonly attribute DOMTokenList sizes;
};
HTMLLinkElement includes LinkStyle;

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLLinkElement {
  [CEReactions, SetterThrows, Pure]
           attribute DOMString charset;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString rev;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString target;
};

// https://w3c.github.io/webappsec/specs/subresourceintegrity/#htmllinkelement-1
partial interface HTMLLinkElement {
  [CEReactions, SetterThrows]
  attribute DOMString integrity;
};

//https://w3c.github.io/preload/
partial interface HTMLLinkElement {
  [SetterThrows, Pure]
           attribute DOMString as;
};
