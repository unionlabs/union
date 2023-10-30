/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-object-element
 * http://www.whatwg.org/specs/web-apps/current-work/#HTMLObjectElement-partial
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-object-element
[HTMLConstructor, NeedResolve]
interface HTMLObjectElement : HTMLElement {
  [CEReactions, Pure, SetterThrows]
           attribute DOMString data;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString type;
  [CEReactions, Pure, SetterThrows]
           attribute boolean typeMustMatch;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString name;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString useMap;
  [Pure]
  readonly attribute HTMLFormElement? form;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString width;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString height;
  // Not pure: can trigger about:blank instantiation
  [NeedsSubjectPrincipal]
  readonly attribute Document? contentDocument;
  // Not pure: can trigger about:blank instantiation
  [NeedsSubjectPrincipal]
  readonly attribute WindowProxy? contentWindow;

  readonly attribute boolean willValidate;
  readonly attribute ValidityState validity;
  [Throws]
  readonly attribute DOMString validationMessage;
  boolean checkValidity();
  boolean reportValidity();
  undefined setCustomValidity(DOMString error);
};

// http://www.whatwg.org/specs/web-apps/current-work/#HTMLObjectElement-partial
partial interface HTMLObjectElement {
  [CEReactions, Pure, SetterThrows]
           attribute DOMString align;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString archive;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString code;
  [CEReactions, Pure, SetterThrows]
           attribute boolean declare;
  [CEReactions, Pure, SetterThrows]
           attribute unsigned long hspace;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString standby;
  [CEReactions, Pure, SetterThrows]
           attribute unsigned long vspace;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString codeBase;
  [CEReactions, Pure, SetterThrows]
           attribute DOMString codeType;

  [CEReactions, TreatNullAs=EmptyString, Pure, SetterThrows]
           attribute DOMString border;
};

partial interface HTMLObjectElement {
  // GetSVGDocument
  [NeedsSubjectPrincipal]
  Document? getSVGDocument();
};
