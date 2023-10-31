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

interface SVGFETurbulenceElement : SVGElement {

  // Turbulence Types
  const unsigned short SVG_TURBULENCE_TYPE_UNKNOWN = 0;
  const unsigned short SVG_TURBULENCE_TYPE_FRACTALNOISE = 1;
  const unsigned short SVG_TURBULENCE_TYPE_TURBULENCE = 2;

  // Stitch Options
  const unsigned short SVG_STITCHTYPE_UNKNOWN = 0;
  const unsigned short SVG_STITCHTYPE_STITCH = 1;
  const unsigned short SVG_STITCHTYPE_NOSTITCH = 2;

  [Constant]
  readonly attribute SVGAnimatedNumber baseFrequencyX;
  [Constant]
  readonly attribute SVGAnimatedNumber baseFrequencyY;
  [Constant]
  readonly attribute SVGAnimatedInteger numOctaves;
  [Constant]
  readonly attribute SVGAnimatedNumber seed;
  [Constant]
  readonly attribute SVGAnimatedEnumeration stitchTiles;
  [Constant]
  readonly attribute SVGAnimatedEnumeration type;
};

SVGFETurbulenceElement includes SVGFilterPrimitiveStandardAttributes;
