/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/
 */

enum CSSStyleSheetParsingMode {
  "author",
  "user",
  "agent"
};

interface CSSStyleSheet : StyleSheet {
  [Pure]
  readonly attribute CSSRule? ownerRule;
  [Throws, NeedsSubjectPrincipal]
  readonly attribute CSSRuleList cssRules;
  [ChromeOnly, BinaryName="parsingModeDOM"]
  readonly attribute CSSStyleSheetParsingMode parsingMode;
  [Throws, NeedsSubjectPrincipal]
  unsigned long insertRule(DOMString rule, optional unsigned long index = 0);
  [Throws, NeedsSubjectPrincipal]
  undefined deleteRule(unsigned long index);
};
