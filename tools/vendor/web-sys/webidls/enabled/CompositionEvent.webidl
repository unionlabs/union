/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * https://w3c.github.io/uievents/#interface-compositionevent
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor(DOMString type, optional CompositionEventInit eventInitDict)]
interface CompositionEvent : UIEvent
{
  readonly attribute DOMString? data;
  // locale is currently non-standard
  readonly attribute DOMString  locale;

 /**
  * ranges is trying to expose TextRangeArray in Gecko so a
  * js-plugin couble be able to know the clauses information
  */
  [ChromeOnly,Cached,Pure]
  readonly attribute sequence<TextClause> ranges;
};

dictionary CompositionEventInit : UIEventInit {
  DOMString data = "";
};

partial interface CompositionEvent
{
  undefined initCompositionEvent(DOMString typeArg,
                            optional boolean canBubbleArg = false,
                            optional boolean cancelableArg = false,
                            optional Window? viewArg = null,
                            optional DOMString? dataArg = null,
                            optional DOMString localeArg = "");
};
