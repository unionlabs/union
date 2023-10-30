/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is:
 * https://html.spec.whatwg.org/multipage/interaction.html#the-datatransferitemlist-interface
 */

interface DataTransferItemList {
  readonly attribute unsigned long length;
  getter DataTransferItem (unsigned long index);
  [Throws, NeedsSubjectPrincipal]
  DataTransferItem? add(DOMString data, DOMString type);
  [Throws, NeedsSubjectPrincipal]
  DataTransferItem? add(File data);
  [Throws, NeedsSubjectPrincipal]
  undefined remove(unsigned long index);
  [Throws, NeedsSubjectPrincipal]
  undefined clear();
};
