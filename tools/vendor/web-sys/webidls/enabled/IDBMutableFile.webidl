/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

[Exposed=Window]
interface IDBMutableFile : EventTarget {
  readonly attribute DOMString name;
  readonly attribute DOMString type;

  readonly attribute IDBDatabase database;

  [Throws, Deprecated="IDBMutableFileOpen"]
  IDBFileHandle open(optional FileMode mode = "readonly");

  [Throws, UseCounter]
  DOMRequest getFile();

  attribute EventHandler onabort;
  attribute EventHandler onerror;
};
