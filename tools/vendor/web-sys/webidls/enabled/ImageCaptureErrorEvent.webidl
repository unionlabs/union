/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/dap/raw-file/default/media-stream-capture/ImageCapture.html
 *
 * Copyright © 2012-2014 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved.
 * W3C liability, trademark and document use rules apply.
 */

[Pref="dom.imagecapture.enabled",
Constructor(DOMString type, optional ImageCaptureErrorEventInit imageCaptureErrorInitDict)]
interface ImageCaptureErrorEvent : Event {
  readonly attribute ImageCaptureError? imageCaptureError;
};

dictionary ImageCaptureErrorEventInit : EventInit {
  ImageCaptureError? imageCaptureError = null;
};

[NoInterfaceObject]
interface ImageCaptureError {
  const unsigned short FRAME_GRAB_ERROR = 1;
  const unsigned short SETTINGS_ERROR = 2;
  const unsigned short PHOTO_ERROR = 3;
  const unsigned short ERROR_UNKNOWN = 4;
  readonly attribute unsigned short code;
  readonly attribute DOMString message;
};

