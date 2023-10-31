/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[NoInterfaceObject]
interface ScrollBoxObject : BoxObject {

  /**
   * Scroll to the given coordinates, in css pixels.
   * (0,0) will put the top left corner of the scrolled element's padding-box
   * at the top left corner of the scrollport (which is its inner-border-box).
   * Values will be clamped to legal values.
   */
  [Throws]
  undefined scrollTo(long x, long y);

  /**
   * Scroll the given amount of device pixels to the right and down.
   * Values will be clamped to make the resuling position legal.
   */
  [Throws]
  undefined scrollBy(long dx, long dy);
  [Throws]
  undefined scrollByIndex(long dindexes);
  [Throws]
  undefined scrollToElement(Element child);

  /**
   * Get the current scroll position in css pixels.
   * @see scrollTo for the definition of x and y.
   */
  [Pure, Throws]
  readonly attribute long positionX;
  [Pure, Throws]
  readonly attribute long positionY;
  [Pure, Throws]
  readonly attribute long scrolledWidth;
  [Pure, Throws]
  readonly attribute long scrolledHeight;

  [Throws]
  undefined ensureElementIsVisible(Element child);
};
