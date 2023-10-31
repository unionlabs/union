/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/cssom/#the-cssrule-interface
 * https://drafts.csswg.org/css-animations/#interface-cssrule
 * https://drafts.csswg.org/css-counter-styles-3/#extentions-to-cssrule-interface
 * https://drafts.csswg.org/css-conditional-3/#extentions-to-cssrule-interface
 * https://drafts.csswg.org/css-fonts-3/#om-fontfeaturevalues
 */

// https://drafts.csswg.org/cssom/#the-cssrule-interface
interface CSSRule {

  const unsigned short STYLE_RULE = 1;
  const unsigned short CHARSET_RULE = 2; // historical
  const unsigned short IMPORT_RULE = 3;
  const unsigned short MEDIA_RULE = 4;
  const unsigned short FONT_FACE_RULE = 5;
  const unsigned short PAGE_RULE = 6;
  // FIXME: We don't support MARGIN_RULE yet.
  // XXXbz Should we expose the constant anyway?
  // const unsigned short MARGIN_RULE = 9;
  const unsigned short NAMESPACE_RULE = 10;
  readonly attribute unsigned short type;
  attribute DOMString cssText;
  readonly attribute CSSRule? parentRule;
  readonly attribute CSSStyleSheet? parentStyleSheet;
};

// https://drafts.csswg.org/css-animations/#interface-cssrule
partial interface CSSRule {
    const unsigned short KEYFRAMES_RULE = 7;
    const unsigned short KEYFRAME_RULE = 8;
};

// https://drafts.csswg.org/css-counter-styles-3/#extentions-to-cssrule-interface
partial interface CSSRule {
    const unsigned short COUNTER_STYLE_RULE = 11;
};

// https://drafts.csswg.org/css-conditional-3/#extentions-to-cssrule-interface
partial interface CSSRule {
    const unsigned short SUPPORTS_RULE = 12;
};

// Non-standard extension for @-moz-document rules.
partial interface CSSRule {
    [ChromeOnly]
    const unsigned short DOCUMENT_RULE = 13;
};

// https://drafts.csswg.org/css-fonts-3/#om-fontfeaturevalues
partial interface CSSRule {
  const unsigned short FONT_FEATURE_VALUES_RULE = 14;
};
