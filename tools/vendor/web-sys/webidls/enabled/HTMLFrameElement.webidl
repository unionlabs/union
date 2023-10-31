/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#htmlframeelement
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#htmlframeelement
[HTMLConstructor]
interface HTMLFrameElement : HTMLElement {
           [CEReactions, SetterThrows]
           attribute DOMString name;
           [CEReactions, SetterThrows]
           attribute DOMString scrolling;
           [CEReactions, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows]
           attribute DOMString src;
           [CEReactions, SetterThrows]
           attribute DOMString frameBorder;
           [CEReactions, SetterThrows]
           attribute DOMString longDesc;
           [CEReactions, SetterThrows]
           attribute boolean noResize;
  [NeedsSubjectPrincipal]
  readonly attribute Document? contentDocument;
  readonly attribute WindowProxy? contentWindow;

  [CEReactions, TreatNullAs=EmptyString, SetterThrows]
  attribute DOMString marginHeight;
  [CEReactions, TreatNullAs=EmptyString, SetterThrows]
  attribute DOMString marginWidth;
};
