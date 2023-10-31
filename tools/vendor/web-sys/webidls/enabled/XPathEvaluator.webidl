/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[Constructor]
interface mixin XPathEvaluator {
  [NewObject, Throws]
  XPathExpression createExpression(DOMString expression,
                                   optional XPathNSResolver? resolver = null);
  [Pure]
  Node createNSResolver(Node nodeResolver);
  [Throws]
  XPathResult evaluate(DOMString expression,
                       Node contextNode,
                       optional XPathNSResolver? resolver = null,
                       optional unsigned short type = 0 /* XPathResult.ANY_TYPE */,
                       optional object? result = null);
};
