/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/selection-api/#selection-interface
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface Selection {
  readonly attribute Node?         anchorNode;
  readonly attribute unsigned long anchorOffset;
  readonly attribute Node?         focusNode;
  readonly attribute unsigned long focusOffset;
  readonly attribute boolean       isCollapsed;
  /**
   * Returns the number of ranges in the selection.
   */
  readonly attribute unsigned long rangeCount;
  readonly attribute DOMString     type;
  /**
   * Returns the range at the specified index.  Throws if the index is
   * out of range.
   */
  [Throws]
  Range     getRangeAt(unsigned long index);
  /**
   * Adds a range to the current selection.
   */
  [Throws, BinaryName="addRangeJS"]
  undefined      addRange(Range range);
  /**
   * Removes a range from the current selection.
   */
  [Throws]
  undefined      removeRange(Range range);
  /**
   * Removes all ranges from the current selection.
   */
  [Throws]
  undefined      removeAllRanges();
  [Throws, BinaryName="RemoveAllRanges"]
  undefined      empty();
  [Throws, BinaryName="collapseJS"]
  undefined      collapse(Node? node, optional unsigned long offset = 0);
  [Throws, BinaryName="collapseJS"]
  undefined      setPosition(Node? node, optional unsigned long offset = 0);
  [Throws, BinaryName="collapseToStartJS"]
  undefined      collapseToStart();
  [Throws, BinaryName="collapseToEndJS"]
  undefined      collapseToEnd();
  [Throws, BinaryName="extendJS"]
  undefined      extend(Node node, optional unsigned long offset = 0);
  [Throws, BinaryName="setBaseAndExtentJS"]
  undefined      setBaseAndExtent(Node anchorNode,
                             unsigned long anchorOffset,
                             Node focusNode,
                             unsigned long focusOffset);
  [Throws, BinaryName="selectAllChildrenJS"]
  undefined      selectAllChildren(Node node);
  [CEReactions, Throws]
  undefined      deleteFromDocument();
  [Throws]
  boolean   containsNode(Node node,
                         optional boolean allowPartialContainment = false);
  stringifier DOMString ();
};

// Additional methods not currently in the spec
partial interface Selection {
  [Throws]
  undefined modify(DOMString alter, DOMString direction,
              DOMString granularity);
};

// Additional chrome-only methods.
// invalid widl
//interface nsISelectionListener;
partial interface Selection {
  /**
   * A true value means "selection after newline"; false means "selection before
   * newline" when a selection is positioned "between lines".
   */
  [ChromeOnly,Throws]
  attribute boolean interlinePosition;

  [Throws]
  attribute short? caretBidiLevel;

  [ChromeOnly,Throws]
  DOMString  toStringWithFormat(DOMString formatType, unsigned long flags, long wrapColumn);
  [ChromeOnly]
  undefined  addSelectionListener(nsISelectionListener newListener);
  [ChromeOnly]
  undefined  removeSelectionListener(nsISelectionListener listenerToRemove);

  [ChromeOnly,BinaryName="rawType"]
  readonly attribute short selectionType;

  /**
   * Return array of ranges intersecting with the given DOM interval.
   */  
  [ChromeOnly,Throws,Pref="dom.testing.selection.GetRangesForInterval"]
  sequence<Range> GetRangesForInterval(Node beginNode, long beginOffset, Node endNode, long endOffset,
                                       boolean allowAdjacent);

  /**
   * Scrolls a region of the selection, so that it is visible in
   * the scrolled view.
   *
   * @param aRegion the region inside the selection to scroll into view
   *                (see selection region constants defined in
   *                nsISelectionController).
   * @param aIsSynchronous when true, scrolls the selection into view
   *                       before returning. If false, posts a request which
   *                       is processed at some point after the method returns.
   * @param aVPercent how to align the frame vertically.
   * @param aHPercent how to align the frame horizontally.
   */
  [ChromeOnly,Throws]
  undefined scrollIntoView(short aRegion, boolean aIsSynchronous, short aVPercent, short aHPercent);

  /**
   * setColors() sets custom colors for the selection.
   * Currently, this is supported only when the selection type is SELECTION_FIND.
   * Otherwise, throws an exception.
   *
   * @param aForegroundColor     The foreground color of the selection.
   *                             If this is "currentColor", foreground color
   *                             isn't changed by this selection.
   * @param aBackgroundColor     The background color of the selection.
   *                             If this is "transparent", background color is
   *                             never painted.
   * @param aAltForegroundColor  The alternative foreground color of the
   *                             selection.
   *                             If aBackgroundColor doesn't have sufficient
   *                             contrast with its around or foreground color
   *                             if "currentColor" is specified, alternative
   *                             colors are used if it have higher contrast.
   * @param aAltBackgroundColor  The alternative background color of the
   *                             selection.
   */
  [ChromeOnly,Throws]
  undefined setColors(DOMString aForegroundColor, DOMString aBackgroundColor,
                 DOMString aAltForegroundColor, DOMString aAltBackgroundColor);

  /**
   * resetColors() forget the customized colors which were set by setColors().
   */
  [ChromeOnly,Throws]
  undefined resetColors();
};
