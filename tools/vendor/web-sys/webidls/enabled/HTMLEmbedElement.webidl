/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-embed-element
 * http://www.whatwg.org/specs/web-apps/current-work/#HTMLEmbedElement-partial
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-embed-element
[HTMLConstructor, NeedResolve]
interface HTMLEmbedElement : HTMLElement {
  [CEReactions, Pure, SetterThrows]
           attribute DOMString src;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString type;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString width;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString height;
};

// http://www.whatwg.org/specs/web-apps/current-work/#HTMLEmbedElement-partial
partial interface HTMLEmbedElement {
  [CEReactions, Pure, SetterThrows]
           attribute DOMString align;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString name;
};

partial interface HTMLEmbedElement {
  // GetSVGDocument
  [NeedsSubjectPrincipal]
  Document? getSVGDocument();
};
