/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBCursorDirection
 */

enum IDBCursorDirection {
    "next",
    "nextunique",
    "prev",
    "prevunique"
};

[Exposed=(Window,Worker)]
interface IDBCursor {
    readonly    attribute (IDBObjectStore or IDBIndex) source;

    [BinaryName="getDirection"]
    readonly    attribute IDBCursorDirection           direction;

    [Throws]
    readonly    attribute any                          key;

    [Throws]
    readonly    attribute any                          primaryKey;

    readonly    attribute IDBRequest                   request;

    [Throws]
    IDBRequest update (any value);

    [Throws]
    undefined       advance ([EnforceRange] unsigned long count);

    [Throws]
    undefined       continue (optional any key);

    [Throws]
    undefined       continuePrimaryKey(any key, any primaryKey);

    [Throws]
    IDBRequest delete ();
};

[Exposed=(Window,Worker)]
interface IDBCursorWithValue : IDBCursor {
    [Throws]
    readonly    attribute any value;
};
