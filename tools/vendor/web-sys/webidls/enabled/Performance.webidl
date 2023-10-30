/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://w3c.github.io/hr-time/
 *
 * Copyright © 2015 W3C® (MIT, ERCIM, Keio, Beihang).
 * W3C liability, trademark and document use rules apply.
 */

typedef sequence <PerformanceEntry> PerformanceEntryList;

[Exposed=(Window,Worker)]
interface Performance : EventTarget {
  [DependsOn=DeviceState, Affects=Nothing]
  DOMHighResTimeStamp now();

  [Constant]
  readonly attribute DOMHighResTimeStamp timeOrigin;
};

[Exposed=Window]
partial interface Performance {
  [Constant]
  readonly attribute PerformanceTiming timing;
  [Constant]
  readonly attribute PerformanceNavigation navigation;

  [Default] object toJSON();
};

// http://www.w3.org/TR/performance-timeline/#sec-window.performance-attribute
[Exposed=(Window,Worker)]
partial interface Performance {
  PerformanceEntryList getEntries();
  PerformanceEntryList getEntriesByType(DOMString entryType);
  PerformanceEntryList getEntriesByName(DOMString name, optional DOMString
    entryType);
};

// http://www.w3.org/TR/resource-timing/#extensions-performance-interface
[Exposed=(Window,Worker)]
partial interface Performance {
  undefined clearResourceTimings();
  undefined setResourceTimingBufferSize(unsigned long maxSize);
  attribute EventHandler onresourcetimingbufferfull;
};

// http://www.w3.org/TR/user-timing/
[Exposed=(Window,Worker)]
partial interface Performance {
  [Throws]
  undefined mark(DOMString markName);
  undefined clearMarks(optional DOMString markName);
  [Throws]
  undefined measure(DOMString measureName, optional DOMString startMark, optional DOMString endMark);
  undefined clearMeasures(optional DOMString measureName);
};
