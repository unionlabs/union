/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/SVG2/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface SVGFESpotLightElement : SVGElement {
  [Constant]
  readonly attribute SVGAnimatedNumber x;
  [Constant]
  readonly attribute SVGAnimatedNumber y;
  [Constant]
  readonly attribute SVGAnimatedNumber z;
  [Constant]
  readonly attribute SVGAnimatedNumber pointsAtX;
  [Constant]
  readonly attribute SVGAnimatedNumber pointsAtY;
  [Constant]
  readonly attribute SVGAnimatedNumber pointsAtZ;
  [Constant]
  readonly attribute SVGAnimatedNumber specularExponent;
  [Constant]
  readonly attribute SVGAnimatedNumber limitingConeAngle;
};
