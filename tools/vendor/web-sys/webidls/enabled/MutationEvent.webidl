/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/2006/webapi/DOM-Level-3-Events/html/DOM3-Events.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */
interface MutationEvent : Event
{
  const unsigned short MODIFICATION = 1;
  const unsigned short ADDITION     = 2;
  const unsigned short REMOVAL      = 3;
  [ChromeOnly]
  const unsigned short SMIL         = 4;

  readonly attribute Node?          relatedNode;
  readonly attribute DOMString      prevValue;
  readonly attribute DOMString      newValue;
  readonly attribute DOMString      attrName;
  readonly attribute unsigned short attrChange;

  [Throws]
  undefined initMutationEvent(DOMString type,
                         optional boolean canBubble = false,
                         optional boolean cancelable = false,
                         optional Node? relatedNode = null,
                         optional DOMString prevValue = "",
                         optional DOMString newValue = "",
                         optional DOMString attrName = "",
                         optional unsigned short attrChange = 0);
};
