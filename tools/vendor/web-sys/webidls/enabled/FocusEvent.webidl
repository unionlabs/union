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

[Constructor(DOMString typeArg, optional FocusEventInit focusEventInitDict)]
interface FocusEvent : UIEvent {
  // Introduced in DOM Level 3:
  readonly attribute EventTarget?   relatedTarget;
};

dictionary FocusEventInit : UIEventInit {
    EventTarget? relatedTarget = null;
};
