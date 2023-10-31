/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/**
 * These objects support visualization of flex containers by the
 * dev tools.
 */

[ChromeOnly]
interface Flex
{
  sequence<FlexLine> getLines();
};

/**
 * Lines with items that have been shrunk are shrinking; with items
 * that have grown are growing, and all others are unchanged.
 */
enum FlexLineGrowthState { "unchanged", "shrinking", "growing" };

[ChromeOnly]
interface FlexLine
{
  readonly attribute FlexLineGrowthState growthState;
  readonly attribute double crossStart;
  readonly attribute double crossSize;

  // firstBaselineOffset measures from flex-start edge.
  readonly attribute double firstBaselineOffset;

  // lastBaselineOffset measures from flex-end edge.
  readonly attribute double lastBaselineOffset;

  /**
   * getItems() returns FlexItems only for the Elements in this Flex
   * container -- ignoring struts and abs-pos Elements.
   */
  sequence<FlexItem> getItems();
};

[ChromeOnly]
interface FlexItem
{
  readonly attribute Node? node;
  readonly attribute double mainBaseSize;
  readonly attribute double mainDeltaSize;
  readonly attribute double mainMinSize;
  readonly attribute double mainMaxSize;
  readonly attribute double crossMinSize;
  readonly attribute double crossMaxSize;
};
