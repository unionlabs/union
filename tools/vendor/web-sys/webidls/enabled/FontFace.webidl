/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/css-font-loading/#fontface-interface
 *
 * Copyright © 2014 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

typedef (ArrayBuffer or ArrayBufferView) BinaryData;

dictionary FontFaceDescriptors {
  DOMString style = "normal";
  DOMString weight = "normal";
  DOMString stretch = "normal";
  DOMString unicodeRange = "U+0-10FFFF";
  DOMString variant = "normal";
  DOMString featureSettings = "normal";
  DOMString variationSettings = "normal";
  DOMString display = "auto";
};

enum FontFaceLoadStatus { "unloaded", "loading", "loaded", "error" };

// Bug 1072107 is for exposing this in workers.
// [Exposed=(Window,Worker)]
[Constructor(DOMString family,
             (DOMString or BinaryData) source,
             optional FontFaceDescriptors descriptors),
 Pref="layout.css.font-loading-api.enabled"]
interface FontFace {
  [SetterThrows] attribute DOMString family;
  [SetterThrows] attribute DOMString style;
  [SetterThrows] attribute DOMString weight;
  [SetterThrows] attribute DOMString stretch;
  [SetterThrows] attribute DOMString unicodeRange;
  [SetterThrows] attribute DOMString variant;
  [SetterThrows] attribute DOMString featureSettings;
  [SetterThrows, Pref="layout.css.font-variations.enabled"] attribute DOMString variationSettings;
  [SetterThrows, Pref="layout.css.font-display.enabled"] attribute DOMString display;

  readonly attribute FontFaceLoadStatus status;

  [Throws]
  Promise<FontFace> load();

  [Throws]
  readonly attribute Promise<FontFace> loaded;
};
