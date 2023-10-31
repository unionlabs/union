/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://svgwg.org/svg2-draft/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface SVGStringList {
  readonly attribute unsigned long length;
  readonly attribute unsigned long numberOfItems;

  undefined clear();
  [Throws]
  DOMString initialize(DOMString newItem);
  [Throws]
  DOMString getItem(unsigned long index);
  getter DOMString(unsigned long index);
  [Throws]
  DOMString insertItemBefore(DOMString newItem, unsigned long index);
  [Throws]
  DOMString replaceItem(DOMString newItem, unsigned long index);
  [Throws]
  DOMString removeItem(unsigned long index);
  [Throws]
  DOMString appendItem(DOMString newItem);
  //setter undefined (unsigned long index, DOMString newItem);
};
