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

[Constructor(DOMString type, optional EventInit eventInitDict),
 Exposed=(Window,Worker,System), ProbablyShortLivingWrapper]
interface Event {
  [Pure]
  readonly attribute DOMString type;
  [Pure, BindingAlias="srcElement"]
  readonly attribute EventTarget? target;
  [Pure]
  readonly attribute EventTarget? currentTarget;

  sequence<EventTarget> composedPath();

  const unsigned short NONE = 0;
  const unsigned short CAPTURING_PHASE = 1;
  const unsigned short AT_TARGET = 2;
  const unsigned short BUBBLING_PHASE = 3;
  [Pure]
  readonly attribute unsigned short eventPhase;

  undefined stopPropagation();
  undefined stopImmediatePropagation();

  [Pure]
  readonly attribute boolean bubbles;
  [Pure]
  readonly attribute boolean cancelable;
  [NeedsCallerType]
  undefined preventDefault();
  [Pure, NeedsCallerType]
  readonly attribute boolean defaultPrevented;
  [ChromeOnly, Pure]
  readonly attribute boolean defaultPreventedByChrome;
  [ChromeOnly, Pure]
  readonly attribute boolean defaultPreventedByContent;
  [Pure]
  readonly attribute boolean composed;

  [Unforgeable, Pure]
  readonly attribute boolean isTrusted;
  [Pure]
  readonly attribute DOMHighResTimeStamp timeStamp;

  undefined initEvent(DOMString type,
                 optional boolean bubbles = false,
                 optional boolean cancelable = false);
  attribute boolean cancelBubble;
};

dictionary EventInit {
  boolean bubbles = false;
  boolean cancelable = false;
  boolean composed = false;
};
