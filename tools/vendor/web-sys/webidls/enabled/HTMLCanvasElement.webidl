/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-canvas-element
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */
/* TODO
interface nsISupports;
interface Variant;
*/

[HTMLConstructor]
interface HTMLCanvasElement : HTMLElement {
  [CEReactions, Pure, SetterThrows]
           attribute unsigned long width;
  [CEReactions, Pure, SetterThrows]
           attribute unsigned long height;

  [Throws]
  nsISupports? getContext(DOMString contextId, optional any contextOptions = null);

  [Throws, NeedsSubjectPrincipal]
  DOMString toDataURL(optional DOMString type = "",
                      optional any encoderOptions);
  [Throws, NeedsSubjectPrincipal]
  undefined toBlob(BlobCallback _callback,
              optional DOMString type = "",
              optional any encoderOptions);
};

// For OffscreenCanvas
// Reference: https://wiki.whatwg.org/wiki/OffscreenCanvas
partial interface HTMLCanvasElement {
  [Pref="gfx.offscreencanvas.enabled", Throws]
  OffscreenCanvas transferControlToOffscreen();
};

callback BlobCallback = undefined(Blob? blob);
