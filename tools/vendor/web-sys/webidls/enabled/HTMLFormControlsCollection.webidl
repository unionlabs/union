/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#htmlformcontrolscollection
 *
 * © Copyright 2004-2013 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface HTMLFormControlsCollection : HTMLCollection {
  // inherits length and item()
  /* legacycaller */ getter (RadioNodeList or Element)? namedItem(DOMString name); // shadows inherited namedItem()
};
