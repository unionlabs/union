/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/css3-animations/#animation-events-
 * http://dev.w3.org/csswg/css3-animations/#animation-events-
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional AnimationEventInit eventInitDict)]
interface AnimationEvent : Event {
  readonly attribute DOMString animationName;
  readonly attribute float     elapsedTime;
  readonly attribute DOMString pseudoElement;
};

dictionary AnimationEventInit : EventInit {
  DOMString animationName = "";
  float elapsedTime = 0;
  DOMString pseudoElement = "";
};
