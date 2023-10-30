/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dom.spec.whatwg.org/#interface-parentnode
 */

interface mixin ParentNode {
  [Constant]
  readonly attribute HTMLCollection children;
  [Pure]
  readonly attribute Element? firstElementChild;
  [Pure]
  readonly attribute Element? lastElementChild;
  [Pure]
  readonly attribute unsigned long childElementCount;

  [CEReactions, Throws, Unscopable]
  undefined prepend((Node or DOMString)... nodes);
  [CEReactions, Throws, Unscopable]
  undefined append((Node or DOMString)... nodes);
};
