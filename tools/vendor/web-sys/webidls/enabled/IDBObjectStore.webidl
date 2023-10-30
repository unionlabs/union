/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBObjectStore
 */

dictionary IDBObjectStoreParameters {
    (DOMString or sequence<DOMString>)? keyPath = null;
    boolean                             autoIncrement = false;
};

[Exposed=(Window,Worker)]
interface IDBObjectStore {
    [SetterThrows]
    attribute DOMString name;

    [Throws]
    readonly    attribute any            keyPath;

    readonly    attribute DOMStringList  indexNames;
    readonly    attribute IDBTransaction transaction;
    readonly    attribute boolean        autoIncrement;

    [Throws]
    IDBRequest put (any value, optional any key);

    [Throws]
    IDBRequest add (any value, optional any key);

    [Throws]
    IDBRequest delete (any key);

    [Throws]
    IDBRequest get (any key);

    [Throws]
    IDBRequest getKey (any key);

    [Throws]
    IDBRequest clear ();

    [Throws]
    IDBRequest openCursor (optional any range, optional IDBCursorDirection direction = "next");

    [Throws]
    IDBIndex   createIndex (DOMString name, (DOMString or sequence<DOMString>) keyPath, optional IDBIndexParameters optionalParameters = {});

    [Throws]
    IDBIndex   index (DOMString name);

    [Throws]
    undefined       deleteIndex (DOMString indexName);

    [Throws]
    IDBRequest count (optional any key);
};

partial interface IDBObjectStore {
    [Throws]
    IDBRequest getAll (optional any key, optional [EnforceRange] unsigned long limit);

    [Throws]
    IDBRequest getAllKeys (optional any key, optional [EnforceRange] unsigned long limit);

    [Throws]
    IDBRequest openKeyCursor (optional any range, optional IDBCursorDirection direction = "next");
};
