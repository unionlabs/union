/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/FXTF/raw-file/tip/filters/index.html
 *
 * Copyright © 2013 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

interface SVGComponentTransferFunctionElement : SVGElement {
  // Component Transfer Types
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_UNKNOWN = 0;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_IDENTITY = 1;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_TABLE = 2;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_DISCRETE = 3;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_LINEAR = 4;
  const unsigned short SVG_FECOMPONENTTRANSFER_TYPE_GAMMA = 5;

  [Constant]
  readonly attribute SVGAnimatedEnumeration type;
  [Constant]
  readonly attribute SVGAnimatedNumberList tableValues;
  [Constant]
  readonly attribute SVGAnimatedNumber slope;
  [Constant]
  readonly attribute SVGAnimatedNumber intercept;
  [Constant]
  readonly attribute SVGAnimatedNumber amplitude;
  [Constant]
  readonly attribute SVGAnimatedNumber exponent;
  [Constant]
  readonly attribute SVGAnimatedNumber offset;
};
