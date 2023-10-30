/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/2006/webapi/FileAPI/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Constructor,
 Exposed=(DedicatedWorker,SharedWorker)]
interface FileReaderSync {

  // Synchronously return strings

  [Throws]
  ArrayBuffer readAsArrayBuffer(Blob blob);
  [Throws]
  DOMString readAsBinaryString(Blob blob);
  [Throws]
  DOMString readAsText(Blob blob, optional DOMString encoding);
  [Throws]
  DOMString readAsDataURL(Blob blob);
};
