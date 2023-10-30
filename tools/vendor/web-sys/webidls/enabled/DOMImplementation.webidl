/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dom.spec.whatwg.org/#interface-domimplementation
 *
 * Copyright:
 * To the extent possible under law, the editors have waived all copyright and
 * related or neighboring rights to this work.
 */

interface DOMImplementation {
  boolean hasFeature();

  [Throws]
  DocumentType createDocumentType(DOMString qualifiedName, DOMString publicId,
                                  DOMString systemId);
  [Throws]
  Document createDocument(DOMString? namespace,
                          [TreatNullAs=EmptyString] DOMString qualifiedName,
                          optional DocumentType? doctype = null);
  [Throws]
  Document createHTMLDocument(optional DOMString title);
};
