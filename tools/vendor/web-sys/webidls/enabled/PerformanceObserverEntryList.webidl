/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/performance-timeline/#the-performanceobserverentrylist-interface
 */

// XXX should be moved into Performance.webidl.
dictionary PerformanceEntryFilterOptions {
  DOMString name;
  DOMString entryType;
  DOMString initiatorType;
};

[Func="mozilla::dom::DOMPrefs::PerformanceObserverEnabled",
 Exposed=(Window,Worker)]
interface PerformanceObserverEntryList {
  PerformanceEntryList getEntries(optional PerformanceEntryFilterOptions filter);
  PerformanceEntryList getEntriesByType(DOMString entryType);
  PerformanceEntryList getEntriesByName(DOMString name,
                                        optional DOMString entryType);
};

