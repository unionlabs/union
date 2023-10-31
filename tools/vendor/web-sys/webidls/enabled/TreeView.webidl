/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// invalid widl
// interface nsITreeSelection;

[NoInterfaceObject]
interface TreeView
{
  /**
   * The total number of rows in the tree (including the offscreen rows).
   */
  readonly attribute long rowCount;

  /**
   * The selection for this view.
   */
  [SetterThrows]
  attribute nsITreeSelection? selection;

  /** 
   * A whitespace delimited list of properties.  For each property X the view
   * gives back will cause the pseudoclasses  ::-moz-tree-cell(x),
   * ::-moz-tree-row(x), ::-moz-tree-twisty(x), ::-moz-tree-image(x),
   * ::-moz-tree-cell-text(x).  to be matched on the pseudoelement
   * ::moz-tree-row.
   */
  [Throws]
  DOMString getRowProperties(long row);

  /**
   * A whitespace delimited list of properties for a given cell.  Each
   * property, x, that the view gives back will cause the pseudoclasses
   *  ::-moz-tree-cell(x), ::-moz-tree-row(x), ::-moz-tree-twisty(x),
   *  ::-moz-tree-image(x), ::-moz-tree-cell-text(x). to be matched on the
   *  cell.
   */
  [Throws]
  DOMString getCellProperties(long row, TreeColumn column);
  
  /**
   * Called to get properties to paint a column background.  For shading the sort
   * column, etc.
   */
  DOMString getColumnProperties(TreeColumn column);

  /**
   * Methods that can be used to test whether or not a twisty should be drawn,
   * and if so, whether an open or closed twisty should be used.
   */
  [Throws]
  boolean isContainer(long row);
  [Throws]
  boolean isContainerOpen(long row);
  [Throws]
  boolean isContainerEmpty(long row);
 
  /**
   * isSeparator is used to determine if the row is a separator.
   * A value of true will result in the tree drawing a horizontal separator.
   * The tree uses the ::moz-tree-separator pseudoclass to draw the separator.
   */
  [Throws]
  boolean isSeparator(long row);

  /**
   * Specifies if there is currently a sort on any column. Used mostly by dragdrop
   * to affect drop feedback.
   */
  boolean isSorted();
  
  const short DROP_BEFORE = -1;
  const short DROP_ON = 0;
  const short DROP_AFTER = 1;
  /**
   * Methods used by the drag feedback code to determine if a drag is allowable at
   * the current location. To get the behavior where drops are only allowed on
   * items, such as the mailNews folder pane, always return false when
   * the orientation is not DROP_ON.
   */
  [Throws]
  boolean canDrop(long row, long orientation, DataTransfer? dataTransfer);

  /**
   * Called when the user drops something on this view. The |orientation| param
   * specifies before/on/after the given |row|.
   */
  [Throws]
  undefined drop(long row, long orientation, DataTransfer? dataTransfer);

  /**
   * Methods used by the tree to draw thread lines in the tree.
   * getParentIndex is used to obtain the index of a parent row.
   * If there is no parent row, getParentIndex returns -1.
   */
  [Throws]
  long getParentIndex(long row);

  /**
   * hasNextSibling is used to determine if the row at rowIndex has a nextSibling
   * that occurs *after* the index specified by afterIndex.  Code that is forced
   * to march down the view looking at levels can optimize the march by starting
   * at afterIndex+1.
   */
  [Throws]
  boolean hasNextSibling(long row, long afterIndex);

  /**
   * The level is an integer value that represents
   * the level of indentation.  It is multiplied by the width specified in the 
   * :moz-tree-indentation pseudoelement to compute the exact indendation.
   */
  [Throws]
  long getLevel(long row);

  /**
   * The image path for a given cell. For defining an icon for a cell.
   * If the empty string is returned, the :moz-tree-image pseudoelement
   * will be used.
   */
  [Throws]
  DOMString getImageSrc(long row, TreeColumn column);

  /**
   * The value for a given cell. This method is only called for columns
   * of type other than |text|.
   */
  [Throws]
  DOMString getCellValue(long row, TreeColumn column);

  /**
   * The text for a given cell.  If a column consists only of an image, then
   * the empty string is returned.  
   */
  [Throws]
  DOMString getCellText(long row, TreeColumn column);

  /**
   * Called during initialization to link the view to the front end box object.
   */
  [Throws]
  undefined setTree(TreeBoxObject? tree);
  
  /**
   * Called on the view when an item is opened or closed.
   */
  [Throws]
  undefined toggleOpenState(long row);

  /**
   * Called on the view when a header is clicked.
   */
  [Throws]
  undefined cycleHeader(TreeColumn column);

  /**
   * Should be called from a XUL onselect handler whenever the selection changes.
   */
  undefined selectionChanged();

  /**
   * Called on the view when a cell in a non-selectable cycling column (e.g., unread/flag/etc.) is clicked.
   */
  undefined cycleCell(long row, TreeColumn column);
  
  /**
   * isEditable is called to ask the view if the cell contents are editable.
   * A value of true will result in the tree popping up a text field when 
   * the user tries to inline edit the cell.
   */
  [Throws]
  boolean isEditable(long row, TreeColumn column);

  /**
   * isSelectable is called to ask the view if the cell is selectable.
   * This method is only called if the selection style is |cell| or |text|.
   * XXXvarga shouldn't this be called isCellSelectable?
   */
  [Throws]
  boolean isSelectable(long row, TreeColumn column);

  /**
   * setCellValue is called when the value of the cell has been set by the user.
   * This method is only called for columns of type other than |text|.
   */
  [Throws]
  undefined setCellValue(long row, TreeColumn column, DOMString value);

  /**
   * setCellText is called when the contents of the cell have been edited by the user.
   */   
  [Throws]
  undefined setCellText(long row, TreeColumn column, DOMString value);

  /**
   * A command API that can be used to invoke commands on the selection.  The tree
   * will automatically invoke this method when certain keys are pressed.  For example,
   * when the DEL key is pressed, performAction will be called with the "delete" string.
   */
  undefined performAction(DOMString action);

  /**
   * A command API that can be used to invoke commands on a specific row.
   */
  undefined performActionOnRow(DOMString action, long row);

  /**
   * A command API that can be used to invoke commands on a specific cell.
   */
  undefined performActionOnCell(DOMString action, long row, TreeColumn column);
};
