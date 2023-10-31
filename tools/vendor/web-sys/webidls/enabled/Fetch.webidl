/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://fetch.spec.whatwg.org/
 */

typedef object JSON;
typedef (Blob or BufferSource or FormData or URLSearchParams or USVString or ReadableStream) BodyInit;

[Exposed=(Window,Worker)]
interface mixin Body {
  readonly attribute boolean bodyUsed;
  [Throws]
  Promise<ArrayBuffer> arrayBuffer();
  [Throws]
  Promise<Blob> blob();
  [Throws]
  Promise<FormData> formData();
  [Throws]
  Promise<JSON> json();
  [Throws]
  Promise<USVString> text();
  readonly attribute ReadableStream? body;
};

// These are helper dictionaries for the parsing of a
// getReader().read().then(data) parsing.
// See more about how these 2 helpers are used in
// dom/fetch/FetchStreamReader.cpp
dictionary FetchReadableStreamReadDataDone {
  boolean done = false;
};

dictionary FetchReadableStreamReadDataArray {
  Uint8Array value;
};
