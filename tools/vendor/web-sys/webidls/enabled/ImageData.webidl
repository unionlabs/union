/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/multipage/the-canvas-element.html#imagedata
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and Opera Software ASA.
 * You are granted a license to use, reproduce and create derivative works of this document.
 */

[Constructor(unsigned long sw, unsigned long sh),
 Constructor(Uint8ClampedArray data, unsigned long sw, optional unsigned long sh),
 Exposed=(Window,Worker)]
interface ImageData {
 [Constant]
 readonly attribute unsigned long width;
 [Constant]
 readonly attribute unsigned long height;
 [Constant, StoreInSlot]
 readonly attribute Uint8ClampedArray data;
};
