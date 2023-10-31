/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/SVG2/pservers.html#InterfaceSVGPatternElement
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface SVGPatternElement : SVGElement {
  [Constant]
  readonly attribute SVGAnimatedEnumeration patternUnits;
  [Constant]
  readonly attribute SVGAnimatedEnumeration patternContentUnits;
  [Constant]
  readonly attribute SVGAnimatedTransformList patternTransform;
  [Constant]
  readonly attribute SVGAnimatedLength x;
  [Constant]
  readonly attribute SVGAnimatedLength y;
  [Constant]
  readonly attribute SVGAnimatedLength width;
  [Constant]
  readonly attribute SVGAnimatedLength height;
};

SVGPatternElement includes SVGFitToViewBox;
SVGPatternElement includes SVGURIReference;
