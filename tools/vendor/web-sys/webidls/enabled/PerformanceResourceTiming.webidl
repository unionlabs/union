/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/resource-timing/#performanceresourcetiming
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Exposed=(Window,Worker)]
interface PerformanceResourceTiming : PerformanceEntry
{
  readonly attribute DOMString initiatorType;
  readonly attribute DOMString nextHopProtocol;

  readonly attribute DOMHighResTimeStamp workerStart;

  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp redirectStart;
  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp redirectEnd;

  readonly attribute DOMHighResTimeStamp fetchStart;

  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp domainLookupStart;
  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp domainLookupEnd;
  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp connectStart;
  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp connectEnd;
  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp secureConnectionStart;
  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp requestStart;
  [NeedsSubjectPrincipal]
  readonly attribute DOMHighResTimeStamp responseStart;

  readonly attribute DOMHighResTimeStamp responseEnd;

  [NeedsSubjectPrincipal]
  readonly attribute unsigned long long transferSize;
  [NeedsSubjectPrincipal]
  readonly attribute unsigned long long encodedBodySize;
  [NeedsSubjectPrincipal]
  readonly attribute unsigned long long decodedBodySize;

  // TODO: Use FrozenArray once available. (Bug 1236777)
  // readonly attribute FrozenArray<PerformanceServerTiming> serverTiming;
  [SecureContext, Frozen, Cached, Pure, NeedsSubjectPrincipal]
  readonly attribute sequence<PerformanceServerTiming> serverTiming;

  [Default] object toJSON();
};
