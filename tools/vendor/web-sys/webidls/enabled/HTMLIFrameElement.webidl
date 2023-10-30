/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-iframe-element
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

[HTMLConstructor]
interface HTMLIFrameElement : HTMLElement {
  [CEReactions, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows, Pure]
           attribute DOMString src;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString srcdoc;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString name;
  [PutForwards=value] readonly attribute DOMTokenList sandbox;
           // attribute boolean seamless;
  [CEReactions, SetterThrows, Pure]
           attribute boolean allowFullscreen;
  [CEReactions, SetterThrows, Pure]
           attribute boolean allowPaymentRequest;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString width;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString height;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString referrerPolicy;
  [NeedsSubjectPrincipal]
  readonly attribute Document? contentDocument;
  readonly attribute WindowProxy? contentWindow;
};

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLIFrameElement {
  [CEReactions, SetterThrows, Pure]
           attribute DOMString align;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString scrolling;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString frameBorder;
  [CEReactions, SetterThrows, Pure]
           attribute DOMString longDesc;

  [CEReactions, TreatNullAs=EmptyString, SetterThrows, Pure]
           attribute DOMString marginHeight;
  [CEReactions, TreatNullAs=EmptyString, SetterThrows, Pure]
           attribute DOMString marginWidth;
};

partial interface HTMLIFrameElement {
  // GetSVGDocument
  [NeedsSubjectPrincipal]
  Document? getSVGDocument();
};

HTMLIFrameElement includes BrowserElement;
