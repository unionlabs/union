/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBRequest
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBRequestReadyState
 */

enum IDBRequestReadyState {
    "pending",
    "done"
};

[Exposed=(Window,Worker)]
interface IDBRequest : EventTarget {
    [Throws]
    readonly    attribute any                  result;

    [Throws]
    readonly    attribute DOMException?        error;

    readonly    attribute (IDBObjectStore or IDBIndex or IDBCursor)? source;
    readonly    attribute IDBTransaction?      transaction;
    readonly    attribute IDBRequestReadyState readyState;

                attribute EventHandler         onsuccess;

                attribute EventHandler         onerror;
};
