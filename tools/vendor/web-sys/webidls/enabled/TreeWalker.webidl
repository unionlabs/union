/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/2012/WD-dom-20120105/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface TreeWalker {
  [Constant]
  readonly attribute Node root;
  [Constant]
  readonly attribute unsigned long whatToShow;
  [Constant]
  readonly attribute NodeFilter? filter;
  [Pure, SetterThrows]
           attribute Node currentNode;

  [Throws]
  Node? parentNode();
  [Throws]
  Node? firstChild();
  [Throws]
  Node? lastChild();
  [Throws]
  Node? previousSibling();
  [Throws]
  Node? nextSibling();
  [Throws]
  Node? previousNode();
  [Throws]
  Node? nextNode();
};
