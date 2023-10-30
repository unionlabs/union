/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

interface XPathExpression {
  // The result specifies a specific result object which may be reused and
  // returned by this method. If this is specified as null or it's not an
  // XPathResult object, a new result object will be constructed and returned.
  [Throws]
  XPathResult evaluate(Node contextNode,
                       optional unsigned short type = 0  /* XPathResult.ANY_TYPE */,
                       optional object? result = null);

  // The result specifies a specific result object which may be reused and
  // returned by this method. If this is specified as null or it's not an
  // XPathResult object, a new result object will be constructed and returned.
  [Throws, ChromeOnly]
  XPathResult evaluateWithContext(Node contextNode,
                                  unsigned long contextPosition,
                                  unsigned long contextSize,
                                  optional unsigned short type = 0  /* XPathResult.ANY_TYPE */,
                                  optional object? result = null);
};
