/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Transition events are defined in:
 * http://www.w3.org/TR/css3-transitions/#transition-events-
 * http://dev.w3.org/csswg/css3-transitions/#transition-events-
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional TransitionEventInit eventInitDict)]
interface TransitionEvent : Event {
  readonly attribute DOMString propertyName;
  readonly attribute float     elapsedTime;
  readonly attribute DOMString pseudoElement;
};

dictionary TransitionEventInit : EventInit {
  DOMString propertyName = "";
  float elapsedTime = 0;
  DOMString pseudoElement = "";
};
