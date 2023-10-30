/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface please see
 * http://dev.w3.org/2006/webapi/DOM-Level-3-Events/html/DOM3-Events.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional WheelEventInit eventInitDict)]
interface WheelEvent : MouseEvent
{
  const unsigned long DOM_DELTA_PIXEL = 0x00;
  const unsigned long DOM_DELTA_LINE  = 0x01;
  const unsigned long DOM_DELTA_PAGE  = 0x02;

  readonly attribute double        deltaX;
  readonly attribute double        deltaY;
  readonly attribute double        deltaZ;
  readonly attribute unsigned long deltaMode;

};

dictionary WheelEventInit : MouseEventInit
{
  double deltaX = 0;
  double deltaY = 0;
  double deltaZ = 0;
  unsigned long deltaMode = 0;
};
