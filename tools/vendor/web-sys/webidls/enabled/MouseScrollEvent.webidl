/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

interface MouseScrollEvent : MouseEvent
{
  const long HORIZONTAL_AXIS = 1;
  const long VERTICAL_AXIS = 2;

  readonly attribute long axis;

  undefined initMouseScrollEvent(DOMString type,
                            optional boolean canBubble = false,
                            optional boolean cancelable = false,
                            optional Window? view = null,
                            optional long detail = 0,
                            optional long screenX = 0,
                            optional long screenY = 0,
                            optional long clientX = 0,
                            optional long clientY = 0,
                            optional boolean ctrlKey = false,
                            optional boolean altKey = false,
                            optional boolean shiftKey = false,
                            optional boolean metaKey = false,
                            optional short button = 0,
                            optional EventTarget? relatedTarget = null,
                            optional long axis = 0);
};
