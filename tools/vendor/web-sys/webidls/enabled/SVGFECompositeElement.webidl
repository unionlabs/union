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

interface SVGFECompositeElement : SVGElement {

  // Composite Operators
  const unsigned short SVG_FECOMPOSITE_OPERATOR_UNKNOWN = 0;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_OVER = 1;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_IN = 2;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_OUT = 3;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_ATOP = 4;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_XOR = 5;
  const unsigned short SVG_FECOMPOSITE_OPERATOR_ARITHMETIC = 6;

  [Constant]
  readonly attribute SVGAnimatedString in1;
  [Constant]
  readonly attribute SVGAnimatedString in2;
  [Constant]
  readonly attribute SVGAnimatedEnumeration operator;
  [Constant]
  readonly attribute SVGAnimatedNumber k1;
  [Constant]
  readonly attribute SVGAnimatedNumber k2;
  [Constant]
  readonly attribute SVGAnimatedNumber k3;
  [Constant]
  readonly attribute SVGAnimatedNumber k4;
};

SVGFECompositeElement includes SVGFilterPrimitiveStandardAttributes;
