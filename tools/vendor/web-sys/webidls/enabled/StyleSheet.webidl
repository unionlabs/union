/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/cssom/
 */

interface StyleSheet {
  [Constant]
  readonly attribute DOMString type;
  [Constant, Throws]
  readonly attribute DOMString? href;
  // Spec says "Node", but it can go null when the node gets a new
  // sheet.  That's also why it's not [Constant]
  [Pure]
  readonly attribute Node? ownerNode;
  [Pure]
  readonly attribute StyleSheet? parentStyleSheet;
  [Pure]
  readonly attribute DOMString? title;
  [Constant, PutForwards=mediaText]
  readonly attribute MediaList media;
  [Pure]
  attribute boolean disabled;
  // The source map URL for this style sheet.  The source map URL can
  // be found in one of two ways.
  //
  // If a SourceMap or X-SourceMap response header is seen, this is
  // the value.  If both are seen, SourceMap is preferred.  Because
  // this relies on the HTTP response, it can change if checked before
  // the response is available -- which is why it is not [Constant].
  //
  // If the style sheet has the special "# sourceMappingURL=" comment,
  // then this is the URL specified there.
  //
  // If the source map URL is not found by either of these methods,
  // then this is an empty string.
  [ChromeOnly, Pure]
  readonly attribute DOMString sourceMapURL;
  // The source URL for this style sheet.  If the style sheet has the
  // special "# sourceURL=" comment, then this is the URL specified
  // there.  If no such comment is found, then this is the empty
  // string.
  [ChromeOnly, Pure]
  readonly attribute DOMString sourceURL;
};
