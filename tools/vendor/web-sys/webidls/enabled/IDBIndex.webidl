/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBIndexParameters
 */

dictionary IDBIndexParameters {
    boolean unique = false;
    boolean multiEntry = false;
    // <null>:   Not locale-aware, uses normal JS sorting.
    // <string>: Always sorted based on the rules of the specified
    //           locale (e.g. "en-US", etc.).
    // "auto":   Sorted by the platform default, may change based on
    //           user agent options.
    DOMString? locale = null;
};

[Exposed=(Window,Worker)]
interface IDBIndex {
    [SetterThrows]
    attribute DOMString name;

    readonly    attribute IDBObjectStore objectStore;

    [Throws]
    readonly    attribute any            keyPath;

    readonly    attribute boolean        multiEntry;
    readonly    attribute boolean        unique;

    // <null>:   Not locale-aware, uses normal JS sorting.
    // <string>: Sorted based on the rules of the specified locale.
    //           Note: never returns "auto", only the current locale.
    [Func="mozilla::dom::IndexedDatabaseManager::ExperimentalFeaturesEnabled"]
    readonly attribute DOMString? locale;

    [Func="mozilla::dom::IndexedDatabaseManager::ExperimentalFeaturesEnabled"]
    readonly attribute boolean isAutoLocale;

    [Throws]
    IDBRequest openCursor (optional any range, optional IDBCursorDirection direction = "next");

    [Throws]
    IDBRequest openKeyCursor (optional any range, optional IDBCursorDirection direction = "next");

    [Throws]
    IDBRequest get (any key);

    [Throws]
    IDBRequest getKey (any key);

    [Throws]
    IDBRequest count (optional any key);
};

partial interface IDBIndex {
    [Throws]
    IDBRequest getAll (optional any key, optional [EnforceRange] unsigned long limit);

    [Throws]
    IDBRequest getAllKeys (optional any key, optional [EnforceRange] unsigned long limit);
};
