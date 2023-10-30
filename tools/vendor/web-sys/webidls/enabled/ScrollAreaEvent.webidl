/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

interface ScrollAreaEvent : UIEvent
{
  readonly attribute float x;
  readonly attribute float y;
  readonly attribute float width;
  readonly attribute float height;

  undefined initScrollAreaEvent(DOMString type,
                           optional boolean canBubble = false,
                           optional boolean cancelable = false,
                           optional Window? view = null,
                           optional long detail = 0,
                           optional float x = 0,
                           optional float y = 0,
                           optional float width = 0,
                           optional float height = 0);
};
