/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://drafts.csswg.org/web-animations/#the-compositeoperation-enumeration
 * https://drafts.csswg.org/web-animations/#dictdef-basepropertyindexedkeyframe
 * https://drafts.csswg.org/web-animations/#dictdef-basekeyframe
 * https://drafts.csswg.org/web-animations/#dictdef-basecomputedkeyframe
 *
 * Copyright © 2016 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

enum CompositeOperation { "replace", "add", "accumulate" };

// The following dictionary types are not referred to by other .webidl files,
// but we use it for manual JS->IDL and IDL->JS conversions in KeyframeEffect's
// implementation.

dictionary BasePropertyIndexedKeyframe {
  (double? or sequence<double?>) offset = [];
  (DOMString or sequence<DOMString>) easing = [];
  (CompositeOperation? or sequence<CompositeOperation?>) composite = [];
};

dictionary BaseKeyframe {
  double? offset = null;
  DOMString easing = "linear";
  CompositeOperation? composite = null;

  // Non-standard extensions

  // Member to allow testing when StyleAnimationValue::ComputeValues fails.
  //
  // Note that we currently only apply this to shorthand properties since
  // it's easier to annotate shorthand property values and because we have
  // only ever observed ComputeValues failing on shorthand values.
  //
  // Bug 1216844 should remove this member since after that bug is fixed we will
  // have a well-defined behavior to use when animation endpoints are not
  // available.
  [ChromeOnly] boolean simulateComputeValuesFailure = false;
};

dictionary BaseComputedKeyframe : BaseKeyframe {
  double computedOffset;
};
