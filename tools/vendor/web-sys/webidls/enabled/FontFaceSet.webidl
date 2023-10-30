/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/csswg/css-font-loading/#FontFaceSet-interface
 *
 * Copyright © 2014 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

// To implement FontFaceSet's iterator until we can use setlike.
dictionary FontFaceSetIteratorResult
{
  required any value;
  required boolean done;
};

// To implement FontFaceSet's iterator until we can use setlike.
[NoInterfaceObject]
interface FontFaceSetIterator {
  [Throws] FontFaceSetIteratorResult next();
};

callback FontFaceSetForEachCallback = undefined (FontFace value, FontFace key, FontFaceSet set);

enum FontFaceSetLoadStatus { "loading", "loaded" };

// Bug 1072762 is for the FontFaceSet constructor.
// [Constructor(sequence<FontFace> initialFaces)]
[Pref="layout.css.font-loading-api.enabled"]
interface FontFaceSet : EventTarget {

  // Emulate setlike behavior until we can use that directly.
  readonly attribute unsigned long size;
  [Throws] undefined add(FontFace font);
  boolean has(FontFace font);
  boolean delete(FontFace font);
  undefined clear();
  [NewObject] FontFaceSetIterator entries();
  // Iterator keys();
  [NewObject, Alias=keys, Alias="@@iterator"] FontFaceSetIterator values();
  [Throws] undefined forEach(FontFaceSetForEachCallback cb, optional any thisArg);

  // -- events for when loading state changes
  attribute EventHandler onloading;
  attribute EventHandler onloadingdone;
  attribute EventHandler onloadingerror;

  // check and start loads if appropriate
  // and fulfill promise when all loads complete
  [NewObject] Promise<sequence<FontFace>> load(DOMString font, optional DOMString text = " ");

  // return whether all fonts in the fontlist are loaded
  // (does not initiate load if not available)
  [Throws] boolean check(DOMString font, optional DOMString text = " ");

  // async notification that font loading and layout operations are done
  [Throws] readonly attribute Promise<undefined> ready;

  // loading state, "loading" while one or more fonts loading, "loaded" otherwise
  readonly attribute FontFaceSetLoadStatus status;
};
