/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dom.spec.whatwg.org/#range
 * http://domparsing.spec.whatwg.org/#dom-range-createcontextualfragment
 * http://dvcs.w3.org/hg/csswg/raw-file/tip/cssom-view/Overview.html#extensions-to-the-range-interface
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor]
interface Range {
  [Throws]
  readonly attribute Node startContainer;
  [Throws]
  readonly attribute unsigned long startOffset;
  [Throws]
  readonly attribute Node endContainer;
  [Throws]
  readonly attribute unsigned long endOffset;
  readonly attribute boolean collapsed;
  [Throws]
  readonly attribute Node commonAncestorContainer;

  [Throws, BinaryName="setStartJS"]
  undefined setStart(Node refNode, unsigned long offset);
  [Throws, BinaryName="setEndJS"]
  undefined setEnd(Node refNode, unsigned long offset);
  [Throws, BinaryName="setStartBeforeJS"]
  undefined setStartBefore(Node refNode);
  [Throws, BinaryName="setStartAfterJS"]
  undefined setStartAfter(Node refNode);
  [Throws, BinaryName="setEndBeforeJS"]
  undefined setEndBefore(Node refNode);
  [Throws, BinaryName="setEndAfterJS"]
  undefined setEndAfter(Node refNode);
  [BinaryName="collapseJS"]
  undefined collapse(optional boolean toStart = false);
  [Throws, BinaryName="selectNodeJS"]
  undefined selectNode(Node refNode);
  [Throws, BinaryName="selectNodeContentsJS"]
  undefined selectNodeContents(Node refNode);

  const unsigned short START_TO_START = 0;
  const unsigned short START_TO_END = 1;
  const unsigned short END_TO_END = 2;
  const unsigned short END_TO_START = 3;
  [Throws]
  short compareBoundaryPoints(unsigned short how, Range sourceRange);
  [CEReactions, Throws]
  undefined deleteContents();
  [CEReactions, Throws]
  DocumentFragment extractContents();
  [CEReactions, Throws]
  DocumentFragment cloneContents();
  [CEReactions, Throws]
  undefined insertNode(Node node);
  [CEReactions, Throws]
  undefined surroundContents(Node newParent);

  Range cloneRange();
  undefined detach();

  [Throws]
  boolean isPointInRange(Node node, unsigned long offset);
  [Throws]
  short comparePoint(Node node, unsigned long offset);

  [Throws]
  boolean intersectsNode(Node node);

  [Throws]
  stringifier;
};

// http://domparsing.spec.whatwg.org/#dom-range-createcontextualfragment
partial interface Range {
  [CEReactions, Throws]
  DocumentFragment createContextualFragment(DOMString fragment);
};

// http://dvcs.w3.org/hg/csswg/raw-file/tip/cssom-view/Overview.html#extensions-to-the-range-interface
partial interface Range {
  DOMRectList? getClientRects();
  DOMRect getBoundingClientRect();
};

dictionary ClientRectsAndTexts {
  required DOMRectList rectList;
  required sequence<DOMString> textList;
};

partial interface Range {
  [ChromeOnly, Throws]
  ClientRectsAndTexts getClientRectsAndTexts();
};
