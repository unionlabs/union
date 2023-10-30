/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/FileAPI/#blob
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

typedef (BufferSource or Blob or USVString) BlobPart;

[Constructor(optional sequence<BlobPart> blobParts,
             optional BlobPropertyBag options),
 Exposed=(Window,Worker)]
interface Blob {

  [GetterThrows]
  readonly attribute unsigned long long size;

  readonly attribute DOMString type;

  //slice Blob into byte-ranged chunks

  [Throws]
  Blob slice([Clamp] optional long long start,
             [Clamp] optional long long end,
             optional DOMString contentType);

  // read from the Blob.
  [NewObject] ReadableStream stream();
  [NewObject] Promise<DOMString> text();
  [NewObject] Promise<ArrayBuffer> arrayBuffer();
};

enum EndingTypes { "transparent", "native" };

dictionary BlobPropertyBag {
  DOMString type = "";
  EndingTypes endings = "transparent";
};
