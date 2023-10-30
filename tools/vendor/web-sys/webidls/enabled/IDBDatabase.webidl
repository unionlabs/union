/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBObjectStoreParameters
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Exposed=(Window,Worker)]
interface IDBDatabase : EventTarget {
    readonly    attribute DOMString          name;
    readonly    attribute unsigned long long version;

    readonly    attribute DOMStringList      objectStoreNames;

    [Throws]
    IDBObjectStore createObjectStore (DOMString name, optional IDBObjectStoreParameters optionalParameters = {});

    [Throws]
    undefined           deleteObjectStore (DOMString name);

    [Throws]
    IDBTransaction transaction ((DOMString or sequence<DOMString>) storeNames,
                                optional IDBTransactionMode mode = "readonly");

    undefined           close ();

                attribute EventHandler       onabort;
                attribute EventHandler       onclose;
                attribute EventHandler       onerror;
                attribute EventHandler       onversionchange;
};

partial interface IDBDatabase {
    [Func="mozilla::dom::IndexedDatabaseManager::ExperimentalFeaturesEnabled"]
    readonly    attribute StorageType        storage;

    [Exposed=Window, Throws, Deprecated="IDBDatabaseCreateMutableFile"]
    IDBRequest createMutableFile (DOMString name, optional DOMString type);
};
