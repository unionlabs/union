/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/fxtf/geometry/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Pref="layout.css.DOMPoint.enabled",
 Constructor(optional unrestricted double x = 0, optional unrestricted double y = 0,
             optional unrestricted double z = 0, optional unrestricted double w = 1)]
interface DOMPointReadOnly {
    [NewObject] static DOMPointReadOnly fromPoint(optional DOMPointInit other);

    readonly attribute unrestricted double x;
    readonly attribute unrestricted double y;
    readonly attribute unrestricted double z;
    readonly attribute unrestricted double w; 

    [Default] object toJSON();
};

[Pref="layout.css.DOMPoint.enabled",
 Constructor(optional unrestricted double x = 0, optional unrestricted double y = 0,
             optional unrestricted double z = 0, optional unrestricted double w = 1)]
interface DOMPoint : DOMPointReadOnly {
    [NewObject] static DOMPoint fromPoint(optional DOMPointInit other);

    inherit attribute unrestricted double x;
    inherit attribute unrestricted double y;
    inherit attribute unrestricted double z;
    inherit attribute unrestricted double w;
};

dictionary DOMPointInit {
    unrestricted double x = 0;
    unrestricted double y = 0;
    unrestricted double z = 0;
    unrestricted double w = 1;
};
