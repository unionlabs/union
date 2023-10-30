/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this file,
* You can obtain one at http://mozilla.org/MPL/2.0/.
*
* The origin of this IDL file is
* http://www.whatwg.org/html/#the-storage-interface
*
* © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
* Opera Software ASA. You are granted a license to use, reproduce
* and create derivative works of this document.
*/

interface Storage {
  [Throws, NeedsSubjectPrincipal]
  readonly attribute unsigned long length;

  [Throws, NeedsSubjectPrincipal]
  DOMString? key(unsigned long index);

  [Throws, NeedsSubjectPrincipal]
  getter DOMString? getItem(DOMString key);

  [Throws, NeedsSubjectPrincipal]
  setter undefined setItem(DOMString key, DOMString value);

  [Throws, NeedsSubjectPrincipal]
  deleter undefined removeItem(DOMString key);

  [Throws, NeedsSubjectPrincipal]
  undefined clear();

  [ChromeOnly]
  readonly attribute boolean isSessionOnly;
};
