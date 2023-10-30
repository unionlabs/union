
/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[NoInterfaceObject]
interface ListBoxObject : BoxObject {

  long getRowCount();
  long getRowHeight();
  long getNumberOfVisibleRows();
  long getIndexOfFirstVisibleRow();

  undefined ensureIndexIsVisible(long rowIndex);
  undefined scrollToIndex(long rowIndex);
  undefined scrollByLines(long numLines);

  Element? getItemAtIndex(long index);
  long getIndexOfItem(Element item);
};
