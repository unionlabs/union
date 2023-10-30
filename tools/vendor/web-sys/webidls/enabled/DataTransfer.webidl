/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is:
 * http://www.whatwg.org/specs/web-apps/current-work/#the-datatransfer-interface
 */

[Constructor]
interface DataTransfer {
           attribute DOMString dropEffect;
           attribute DOMString effectAllowed;

  readonly attribute DataTransferItemList items;

  undefined setDragImage(Element image, long x, long y);

  // ReturnValueNeedsContainsHack on .types because lots of extension
  // code was expecting .contains() back when it was a DOMStringList.
  [Pure, Cached, Frozen, NeedsCallerType, ReturnValueNeedsContainsHack]
  readonly attribute sequence<DOMString> types;
  [Throws, NeedsSubjectPrincipal]
  DOMString getData(DOMString format);
  [Throws, NeedsSubjectPrincipal]
  undefined setData(DOMString format, DOMString data);
  [Throws, NeedsSubjectPrincipal]
  undefined clearData(optional DOMString format);
  [NeedsSubjectPrincipal]
  readonly attribute FileList? files;
};

partial interface DataTransfer {
  [Throws, Pref="dom.input.dirpicker", NeedsSubjectPrincipal]
  Promise<sequence<(File or Directory)>> getFilesAndDirectories();

  [Throws, Pref="dom.input.dirpicker", NeedsSubjectPrincipal]
  Promise<sequence<File>>                getFiles(optional boolean recursiveFlag = false);
};
