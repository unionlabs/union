/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtaone at http://mozilla.org/MPL/2.0/. */

dictionary IDBFileMetadataParameters
{
  boolean size = true;
  boolean lastModified = true;
};

[Exposed=Window]
interface IDBFileHandle : EventTarget
{
  readonly attribute IDBMutableFile? mutableFile;
  // this is deprecated due to renaming in the spec
  readonly attribute IDBMutableFile? fileHandle; // now mutableFile
  readonly attribute FileMode mode;
  readonly attribute boolean active;
  attribute unsigned long long? location;

  [Throws]
  IDBFileRequest? getMetadata(optional IDBFileMetadataParameters parameters = {});
  [Throws]
  IDBFileRequest? readAsArrayBuffer(unsigned long long size);
  [Throws]
  IDBFileRequest? readAsText(unsigned long long size,
                             optional DOMString? encoding = null);

  [Throws]
  IDBFileRequest? write((DOMString or ArrayBuffer or ArrayBufferView or Blob) value);
  [Throws]
  IDBFileRequest? append((DOMString or ArrayBuffer or ArrayBufferView or Blob) value);
  [Throws]
  IDBFileRequest? truncate(optional unsigned long long size);
  [Throws]
  IDBFileRequest? flush();
  [Throws]
  undefined abort();

  attribute EventHandler oncomplete;
  attribute EventHandler onabort;
  attribute EventHandler onerror;
};
