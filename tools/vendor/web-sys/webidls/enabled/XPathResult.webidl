/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Corresponds to http://www.w3.org/TR/2002/WD-DOM-Level-3-XPath-20020208
 */

interface XPathResult {
  // XPathResultType
  const unsigned short ANY_TYPE = 0;
  const unsigned short NUMBER_TYPE = 1;
  const unsigned short STRING_TYPE = 2;
  const unsigned short BOOLEAN_TYPE = 3;
  const unsigned short UNORDERED_NODE_ITERATOR_TYPE = 4;
  const unsigned short ORDERED_NODE_ITERATOR_TYPE = 5;
  const unsigned short UNORDERED_NODE_SNAPSHOT_TYPE = 6;
  const unsigned short ORDERED_NODE_SNAPSHOT_TYPE = 7;
  const unsigned short ANY_UNORDERED_NODE_TYPE = 8;
  const unsigned short FIRST_ORDERED_NODE_TYPE = 9;

  readonly attribute unsigned short resultType;
  [Throws]
  readonly attribute double numberValue;
  [Throws]
  readonly attribute DOMString stringValue;
  [Throws]
  readonly attribute boolean booleanValue;
  [Throws]
  readonly attribute Node? singleNodeValue;
  readonly attribute boolean invalidIteratorState;
  [Throws]
  readonly attribute unsigned long snapshotLength;
  [Throws]
  Node? iterateNext();
  [Throws]
  Node? snapshotItem(unsigned long index);
};
