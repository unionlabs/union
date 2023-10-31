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

interface SVGFEBlendElement : SVGElement {

  // Blend Mode Types
  const unsigned short SVG_FEBLEND_MODE_UNKNOWN = 0;
  const unsigned short SVG_FEBLEND_MODE_NORMAL = 1;
  const unsigned short SVG_FEBLEND_MODE_MULTIPLY = 2;
  const unsigned short SVG_FEBLEND_MODE_SCREEN = 3;
  const unsigned short SVG_FEBLEND_MODE_DARKEN = 4;
  const unsigned short SVG_FEBLEND_MODE_LIGHTEN = 5;
  const unsigned short SVG_FEBLEND_MODE_OVERLAY = 6;
  const unsigned short SVG_FEBLEND_MODE_COLOR_DODGE = 7;
  const unsigned short SVG_FEBLEND_MODE_COLOR_BURN = 8;
  const unsigned short SVG_FEBLEND_MODE_HARD_LIGHT = 9;
  const unsigned short SVG_FEBLEND_MODE_SOFT_LIGHT = 10;
  const unsigned short SVG_FEBLEND_MODE_DIFFERENCE = 11;
  const unsigned short SVG_FEBLEND_MODE_EXCLUSION = 12;
  const unsigned short SVG_FEBLEND_MODE_HUE = 13;
  const unsigned short SVG_FEBLEND_MODE_SATURATION = 14;
  const unsigned short SVG_FEBLEND_MODE_COLOR = 15;
  const unsigned short SVG_FEBLEND_MODE_LUMINOSITY = 16;
  [Constant]
  readonly attribute SVGAnimatedString in1;
  [Constant]
  readonly attribute SVGAnimatedString in2;
  [Constant]
  readonly attribute SVGAnimatedEnumeration mode;
};

SVGFEBlendElement includes SVGFilterPrimitiveStandardAttributes;
