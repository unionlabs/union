/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// http://dev.w3.org/csswg/cssom/#the-medialist-interface

interface MediaList {
  // Bug 824857: no support for stringifier attributes yet.
  //   [TreatNullAs=EmptyString]
  // stringifier attribute DOMString        mediaText;

  // Bug 824857 should remove this.
  stringifier;

  [TreatNullAs=EmptyString]
           attribute DOMString        mediaText;

  readonly attribute unsigned long    length;
  getter DOMString?  item(unsigned long index);
  [Throws]
  undefined               deleteMedium(DOMString oldMedium);
  [Throws]
  undefined               appendMedium(DOMString newMedium);
};
