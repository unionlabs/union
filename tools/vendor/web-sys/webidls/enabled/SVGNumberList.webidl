/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/SVG11/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface SVGNumberList {
  readonly attribute unsigned long numberOfItems;
  [Throws]
  undefined clear();
  [Throws]
  SVGNumber initialize(SVGNumber newItem);
  [Throws]
  getter SVGNumber getItem(unsigned long index);
  [Throws]
  SVGNumber insertItemBefore(SVGNumber newItem, unsigned long index);
  [Throws]
  SVGNumber replaceItem(SVGNumber newItem, unsigned long index);
  [Throws]
  SVGNumber removeItem(unsigned long index);
  [Throws]
  SVGNumber appendItem(SVGNumber newItem);
};
