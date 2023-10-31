/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#htmlimageelement
 * http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

/* TODO
interface imgINotificationObserver;
interface imgIRequest;
interface URI;
interface nsIStreamListener;
*/

[NamedConstructor=Image(optional unsigned long width, optional unsigned long height),
 Exposed=Window]
interface HTMLImageElement : HTMLElement {
           [CEReactions, SetterThrows]
           attribute DOMString alt;
           [CEReactions, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows]
           attribute DOMString src;
           [CEReactions, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows]
           attribute DOMString srcset;
           [CEReactions, SetterThrows]
           attribute DOMString? crossOrigin;
           [CEReactions, SetterThrows]
           attribute DOMString useMap;
           [CEReactions, SetterThrows]
           attribute DOMString referrerPolicy;
           [CEReactions, SetterThrows]
           attribute boolean isMap;
           [CEReactions, SetterThrows]
           attribute unsigned long width;
           [CEReactions, SetterThrows]
           attribute unsigned long height;
           [CEReactions, SetterThrows]
           attribute DOMString decoding;
  readonly attribute unsigned long naturalWidth;
  readonly attribute unsigned long naturalHeight;
  readonly attribute boolean complete;
           [NewObject]
           Promise<undefined> decode();
};

// http://www.whatwg.org/specs/web-apps/current-work/#other-elements,-attributes-and-apis
partial interface HTMLImageElement {
           [CEReactions, SetterThrows]
           attribute DOMString name;
           [CEReactions, SetterThrows]
           attribute DOMString align;
           [CEReactions, SetterThrows]
           attribute unsigned long hspace;
           [CEReactions, SetterThrows]
           attribute unsigned long vspace;
           [CEReactions, SetterThrows]
           attribute DOMString longDesc;

  [CEReactions, SetterThrows] attribute [TreatNullAs=EmptyString] DOMString border;
};

// [Update me: not in whatwg spec yet]
// http://picture.responsiveimages.org/#the-img-element
partial interface HTMLImageElement {
           [CEReactions, SetterThrows]
           attribute DOMString sizes;
  readonly attribute DOMString currentSrc;
};
