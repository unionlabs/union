/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://wiki.whatwg.org/wiki/OffscreenCanvas
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// The new ImageBitmapRenderingContext is a canvas rendering context
// which only provides the functionality to replace the canvas's
// contents with the given ImageBitmap. Its context id (the first argument
// to getContext) is "bitmaprenderer".
[Exposed=(Window,Worker)]
interface ImageBitmapRenderingContext {
  // Displays the given ImageBitmap in the canvas associated with this
  // rendering context. Ownership of the ImageBitmap is transferred to
  // the canvas. The caller may not use its reference to the ImageBitmap
  // after making this call. (This semantic is crucial to enable prompt
  // reclamation of expensive graphics resources, rather than relying on
  // garbage collection to do so.)
  //
  // The ImageBitmap conceptually replaces the canvas's bitmap, but
  // it does not change the canvas's intrinsic width or height.
  //
  // The ImageBitmap, when displayed, is clipped to the rectangle
  // defined by the canvas's instrinsic width and height. Pixels that
  // would be covered by the canvas's bitmap which are not covered by
  // the supplied ImageBitmap are rendered transparent black. Any CSS
  // styles affecting the display of the canvas are applied as usual.
  undefined transferFromImageBitmap(ImageBitmap bitmap);

  // Deprecated version of transferFromImageBitmap
  [Deprecated="ImageBitmapRenderingContext_TransferImageBitmap"]
  undefined transferImageBitmap(ImageBitmap bitmap);
};
