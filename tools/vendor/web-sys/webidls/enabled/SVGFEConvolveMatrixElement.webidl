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

interface SVGFEConvolveMatrixElement : SVGElement {

  // Edge Mode Values
  const unsigned short SVG_EDGEMODE_UNKNOWN = 0;
  const unsigned short SVG_EDGEMODE_DUPLICATE = 1;
  const unsigned short SVG_EDGEMODE_WRAP = 2;
  const unsigned short SVG_EDGEMODE_NONE = 3;

  [Constant]
  readonly attribute SVGAnimatedString in1;
  [Constant]
  readonly attribute SVGAnimatedInteger orderX;
  [Constant]
  readonly attribute SVGAnimatedInteger orderY;
  [Constant]
  readonly attribute SVGAnimatedNumberList kernelMatrix;
  [Constant]
  readonly attribute SVGAnimatedNumber divisor;
  [Constant]
  readonly attribute SVGAnimatedNumber bias;
  [Constant]
  readonly attribute SVGAnimatedInteger targetX;
  [Constant]
  readonly attribute SVGAnimatedInteger targetY;
  [Constant]
  readonly attribute SVGAnimatedEnumeration edgeMode;
  [Constant]
  readonly attribute SVGAnimatedNumber kernelUnitLengthX;
  [Constant]
  readonly attribute SVGAnimatedNumber kernelUnitLengthY;
  [Constant]
  readonly attribute SVGAnimatedBoolean preserveAlpha;
};

SVGFEConvolveMatrixElement includes SVGFilterPrimitiveStandardAttributes;
