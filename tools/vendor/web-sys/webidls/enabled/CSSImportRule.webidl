/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/cssom/#cssimportrule
 */

// https://drafts.csswg.org/cssom/#cssimportrule
interface CSSImportRule : CSSRule {
  readonly attribute DOMString href;
  // Per spec, the .media is never null, but in our implementation it can
  // be since stylesheet can be null, and in Stylo, media is derived from
  // the stylesheet.  See <https://bugzilla.mozilla.org/show_bug.cgi?id=1326509>.
  [SameObject, PutForwards=mediaText] readonly attribute MediaList? media;
  // Per spec, the .styleSheet is never null, but in our implementation it can
  // be.  See <https://bugzilla.mozilla.org/show_bug.cgi?id=1326509>.
  [SameObject] readonly attribute CSSStyleSheet? styleSheet;
};
