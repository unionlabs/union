/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBFactory
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

// invalid widl
// interface Principal;

dictionary IDBOpenDBOptions
{
  [EnforceRange] unsigned long long version;
  StorageType storage;
};

/**
 * Interface that defines the indexedDB property on a window.  See
 * http://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBFactory
 * for more information.
 */
[Exposed=(Window,Worker)]
interface IDBFactory {
  [Throws, NeedsCallerType]
  IDBOpenDBRequest
  open(DOMString name,
       [EnforceRange] unsigned long long version);

  [Throws, NeedsCallerType]
  IDBOpenDBRequest
  open(DOMString name,
       optional IDBOpenDBOptions options = {});

  [Throws, NeedsCallerType]
  IDBOpenDBRequest
  deleteDatabase(DOMString name,
                 optional IDBOpenDBOptions options = {});

  [Throws]
  short
  cmp(any first,
      any second);

  [Throws, ChromeOnly, NeedsCallerType]
  IDBOpenDBRequest
  openForPrincipal(Principal principal,
                   DOMString name,
                   [EnforceRange] unsigned long long version);

  [Throws, ChromeOnly, NeedsCallerType]
  IDBOpenDBRequest
  openForPrincipal(Principal principal,
                   DOMString name,
                   optional IDBOpenDBOptions options = {});

  [Throws, ChromeOnly, NeedsCallerType]
  IDBOpenDBRequest
  deleteForPrincipal(Principal principal,
                     DOMString name,
                     optional IDBOpenDBOptions options = {});
};
