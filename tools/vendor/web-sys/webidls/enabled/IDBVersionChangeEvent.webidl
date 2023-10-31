/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/IndexedDB/raw-file/tip/Overview.html#idl-def-IDBVersionChangeEvent
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

dictionary IDBVersionChangeEventInit : EventInit {
    unsigned long long  oldVersion = 0;
    unsigned long long? newVersion = null;
};

[Exposed=(Window,Worker)]
interface IDBVersionChangeEvent : Event {
    constructor(DOMString type,
                optional IDBVersionChangeEventInit eventInitDict = {});

    readonly    attribute unsigned long long  oldVersion;
    readonly    attribute unsigned long long? newVersion;
};
