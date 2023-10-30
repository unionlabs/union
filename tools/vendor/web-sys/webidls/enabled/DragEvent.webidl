/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[Constructor(DOMString type, optional DragEventInit eventInitDict)]
interface DragEvent : MouseEvent
{
  readonly attribute DataTransfer? dataTransfer;

  undefined initDragEvent(DOMString type,
                     optional boolean canBubble = false,
                     optional boolean cancelable = false,
                     optional Window? aView = null,
                     optional long aDetail = 0,
                     optional long aScreenX = 0,
                     optional long aScreenY = 0,
                     optional long aClientX = 0,
                     optional long aClientY = 0,
                     optional boolean aCtrlKey = false,
                     optional boolean aAltKey = false,
                     optional boolean aShiftKey = false,
                     optional boolean aMetaKey = false,
                     optional unsigned short aButton = 0,
                     optional EventTarget? aRelatedTarget = null,
                     optional DataTransfer? aDataTransfer = null);
};

dictionary DragEventInit : MouseEventInit
{
  DataTransfer? dataTransfer = null;
};
