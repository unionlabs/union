/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/
 */

 // Because of getComputedStyle, many CSSStyleDeclaration objects can be
 // short-living.
[ProbablyShortLivingWrapper]
interface CSSStyleDeclaration {
  [CEReactions, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows]
  attribute DOMString cssText;

  readonly attribute unsigned long length;
  getter DOMString item(unsigned long index);

  [Throws, ChromeOnly]
  sequence<DOMString> getCSSImageURLs(DOMString property);

  [Throws]
  DOMString getPropertyValue(DOMString property);
  DOMString getPropertyPriority(DOMString property);
  [CEReactions, NeedsSubjectPrincipal=NonSystem, Throws]
  undefined setProperty(DOMString property, [TreatNullAs=EmptyString] DOMString value, [TreatNullAs=EmptyString] optional DOMString priority = "");
  [CEReactions, Throws]
  DOMString removeProperty(DOMString property);

  readonly attribute CSSRule? parentRule;
};
