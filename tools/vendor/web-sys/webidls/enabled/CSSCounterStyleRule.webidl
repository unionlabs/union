/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/css-counter-styles-3/#the-csscounterstylerule-interface
 */

// https://drafts.csswg.org/css-counter-styles-3/#the-csscounterstylerule-interface
interface CSSCounterStyleRule : CSSRule {
  attribute DOMString name;
  attribute DOMString system;
  attribute DOMString symbols;
  attribute DOMString additiveSymbols;
  attribute DOMString negative;
  attribute DOMString prefix;
  attribute DOMString suffix;
  attribute DOMString range;
  attribute DOMString pad;
  attribute DOMString speakAs;
  attribute DOMString fallback;
};
