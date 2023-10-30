/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/FileAPI/#APIASynch
 *
 * Copyright © 2013 W3C® (MIT, ERCIM, Keio, Beihang), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor,
 Exposed=(Window,Worker,System)]
interface FileReader : EventTarget {
  // async read methods
  [Throws]
  undefined readAsArrayBuffer(Blob blob);
  [Throws]
  undefined readAsBinaryString(Blob filedata);
  [Throws]
  undefined readAsText(Blob blob, optional DOMString label);
  [Throws]
  undefined readAsDataURL(Blob blob);

  undefined abort();

  // states
  const unsigned short EMPTY = 0;
  const unsigned short LOADING = 1;
  const unsigned short DONE = 2;


  readonly attribute unsigned short readyState;

  // File or Blob data
  // bug 858217: readonly attribute (DOMString or ArrayBuffer)? result;
  [Throws]
  readonly attribute any result;

  readonly attribute DOMException? error;

  // event handler attributes
  attribute EventHandler onloadstart;
  attribute EventHandler onprogress;
  attribute EventHandler onload;
  attribute EventHandler onabort;
  attribute EventHandler onerror;
  attribute EventHandler onloadend;
};
