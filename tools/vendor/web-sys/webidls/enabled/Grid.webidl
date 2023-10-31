/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/* These objects support visualization of a css-grid by the dev tools. */

/**
 * Explicit and implicit types apply to tracks, lines, and areas.
 * https://drafts.csswg.org/css-grid/#explicit-grids
 * https://drafts.csswg.org/css-grid/#implicit-grids
 */
enum GridDeclaration { "explicit", "implicit" };

/**
 * Tracks expanded from auto-fill are repeat , auto-fits with elements are
 * also repeat, auto-fits with no elements are removed, other tracks are static.
 */
enum GridTrackState { "static", "repeat", "removed" };

[ChromeOnly]
interface Grid
{
  readonly attribute GridDimension rows;
  readonly attribute GridDimension cols;
  [Cached, Constant]
  readonly attribute sequence<GridArea> areas;
};

[ChromeOnly]
interface GridDimension
{
  readonly attribute GridLines lines;
  readonly attribute GridTracks tracks;
};

[ChromeOnly]
interface GridLines
{
  readonly attribute unsigned long length;

  /**
   * This accessor method allows array-like access to lines.
   * @param index A 0-indexed value.
   */
  getter GridLine? item(unsigned long index);
};

[ChromeOnly]
interface GridLine
{
  /**
   * Names include both explicit names and implicit names, which will be
   * assigned if the line contributes to a named area.
   * https://drafts.csswg.org/css-grid/#implicit-named-lines
   */
  [Cached, Constant]
  readonly attribute sequence<DOMString> names;

  readonly attribute double start;

  /**
   * Breadth is the gap between the start of this line and the start of the
   * next track in flow direction. It primarily is set by use of the -gap
   * properties.
   * https://drafts.csswg.org/css-grid/#gutters
   */
  readonly attribute double breadth;

  readonly attribute GridDeclaration type;

  /**
   * Number is the 1-indexed index of the line in flow order. The
   * first explicit line has number 1, and numbers increment by 1 for
   * each line after that. Lines before the first explicit line
   * have number 0, which is not a valid addressable line number, and
   * should be filtered out by callers.
   */
  readonly attribute unsigned long number;

  /**
   * NegativeNumber is the 1-indexed index of the line in reverse
   * flow order. The last explicit line has negativeNumber -1, and
   * negativeNumbers decrement by 1 for each line before that.
   * Lines after the last explicit line have negativeNumber 0, which
   * is not a valid addressable line number, and should be filtered
   * out by callers.
   */
  readonly attribute long negativeNumber;
};

[ChromeOnly]
interface GridTracks
{
  readonly attribute unsigned long length;

  /**
   * This accessor method allows array-like access to tracks.
   * @param index A 0-indexed value.
   */
  getter GridTrack? item(unsigned long index);
};

[ChromeOnly]
interface GridTrack
{
  readonly attribute double start;
  readonly attribute double breadth;
  readonly attribute GridDeclaration type;
  readonly attribute GridTrackState state;
};

[ChromeOnly]
interface GridArea
{
  readonly attribute DOMString name;
  readonly attribute GridDeclaration type;

  /**
   * These values are 1-indexed line numbers bounding the area.
   */
  readonly attribute unsigned long rowStart;
  readonly attribute unsigned long rowEnd;
  readonly attribute unsigned long columnStart;
  readonly attribute unsigned long columnEnd;
};
