/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/2012/WD-dom-20120105/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional CustomEventInit eventInitDict),
 Exposed=(Window, Worker)]
interface CustomEvent : Event
{
  readonly attribute any detail;

  // initCustomEvent is a Gecko specific deprecated method.
  undefined initCustomEvent(DOMString type,
                       optional boolean canBubble = false,
                       optional boolean cancelable = false,
                       optional any detail = null);
};

dictionary CustomEventInit : EventInit
{
  any detail = null;
};
