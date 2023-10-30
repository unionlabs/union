/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

interface Screen : EventTarget {
  // CSSOM-View
  // http://dev.w3.org/csswg/cssom-view/#the-screen-interface
  [Throws]
  readonly attribute long availWidth;
  [Throws]
  readonly attribute long availHeight;
  [Throws]
  readonly attribute long width;
  [Throws]
  readonly attribute long height;
  [Throws]
  readonly attribute long colorDepth;
  [Throws]
  readonly attribute long pixelDepth;

  [Throws]
  readonly attribute long top;
  [Throws]
  readonly attribute long left;
  [Throws]
  readonly attribute long availTop;
  [Throws]
  readonly attribute long availLeft;
};

// https://w3c.github.io/screen-orientation
partial interface Screen {
  readonly attribute ScreenOrientation orientation;
};

// https://wicg.github.io/media-capabilities/#idl-index
enum ScreenColorGamut {
  "srgb",
  "p3",
  "rec2020",
};

[Func="mozilla::dom::MediaCapabilities::Enabled"]
interface ScreenLuminance {
  readonly attribute double min;
  readonly attribute double max;
  readonly attribute double maxAverage;
};

partial interface Screen {
  [Func="mozilla::dom::MediaCapabilities::Enabled"]
  readonly attribute ScreenColorGamut colorGamut;
  [Func="mozilla::dom::MediaCapabilities::Enabled"]
  readonly attribute ScreenLuminance? luminance;

  [Func="mozilla::dom::MediaCapabilities::Enabled"]
  attribute EventHandler onchange;
};
