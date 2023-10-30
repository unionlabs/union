/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://compat.spec.whatwg.org/#webkitcssmatrix-interface
 */

[Constructor,
 Constructor(DOMString transformList),
 Constructor(WebKitCSSMatrix other),
 Exposed=Window,
 Func="mozilla::dom::WebKitCSSMatrix::FeatureEnabled"]
interface WebKitCSSMatrix : DOMMatrix {
    // Mutable transform methods
    [Throws]
    WebKitCSSMatrix setMatrixValue(DOMString transformList);

    // Immutable transform methods
    WebKitCSSMatrix multiply(WebKitCSSMatrix other);
    [Throws]
    WebKitCSSMatrix inverse();
    WebKitCSSMatrix translate(optional unrestricted double tx = 0,
                              optional unrestricted double ty = 0,
                              optional unrestricted double tz = 0);
    WebKitCSSMatrix scale(optional unrestricted double scaleX = 1,
                          optional unrestricted double scaleY,
                          optional unrestricted double scaleZ = 1);
    WebKitCSSMatrix rotate(optional unrestricted double rotX = 0,
                           optional unrestricted double rotY,
                           optional unrestricted double rotZ);
    WebKitCSSMatrix rotateAxisAngle(optional unrestricted double x = 0,
                                    optional unrestricted double y = 0,
                                    optional unrestricted double z = 0,
                                    optional unrestricted double angle = 0);
    WebKitCSSMatrix skewX(optional unrestricted double sx = 0);
    WebKitCSSMatrix skewY(optional unrestricted double sy = 0);
};
