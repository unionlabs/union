/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/cssom/#cssgroupingrule
 */

// https://drafts.csswg.org/cssom/#cssgroupingrule
interface CSSGroupingRule : CSSRule {
  [SameObject] readonly attribute CSSRuleList cssRules;
  [Throws]
  unsigned long insertRule(DOMString rule, optional unsigned long index = 0);
  [Throws]
  undefined deleteRule(unsigned long index);
};
