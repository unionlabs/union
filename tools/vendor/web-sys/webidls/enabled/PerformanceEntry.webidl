/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3c-test.org/webperf/specs/PerformanceTimeline/#sec-PerformanceEntry-interface
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Exposed=(Window,Worker)]
interface PerformanceEntry
{
  readonly attribute DOMString name;
  readonly attribute DOMString entryType;
  readonly attribute DOMHighResTimeStamp startTime;
  readonly attribute DOMHighResTimeStamp duration;

  [Default] object toJSON();
};
