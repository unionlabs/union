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

[NoInterfaceObject]
interface SVGPathSeg {

  // Path Segment Types
  const unsigned short PATHSEG_UNKNOWN = 0;
  const unsigned short PATHSEG_CLOSEPATH = 1;
  const unsigned short PATHSEG_MOVETO_ABS = 2;
  const unsigned short PATHSEG_MOVETO_REL = 3;
  const unsigned short PATHSEG_LINETO_ABS = 4;
  const unsigned short PATHSEG_LINETO_REL = 5;
  const unsigned short PATHSEG_CURVETO_CUBIC_ABS = 6;
  const unsigned short PATHSEG_CURVETO_CUBIC_REL = 7;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_ABS = 8;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_REL = 9;
  const unsigned short PATHSEG_ARC_ABS = 10;
  const unsigned short PATHSEG_ARC_REL = 11;
  const unsigned short PATHSEG_LINETO_HORIZONTAL_ABS = 12;
  const unsigned short PATHSEG_LINETO_HORIZONTAL_REL = 13;
  const unsigned short PATHSEG_LINETO_VERTICAL_ABS = 14;
  const unsigned short PATHSEG_LINETO_VERTICAL_REL = 15;
  const unsigned short PATHSEG_CURVETO_CUBIC_SMOOTH_ABS = 16;
  const unsigned short PATHSEG_CURVETO_CUBIC_SMOOTH_REL = 17;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_SMOOTH_ABS = 18;
  const unsigned short PATHSEG_CURVETO_QUADRATIC_SMOOTH_REL = 19;

  [Pure]
  readonly attribute unsigned short pathSegType;
  [Pure]
  readonly attribute DOMString pathSegTypeAsLetter;
};

[NoInterfaceObject]
interface SVGPathSegClosePath : SVGPathSeg {
};

[NoInterfaceObject]
interface SVGPathSegMovetoAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
};

[NoInterfaceObject]
interface SVGPathSegMovetoRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
};

[NoInterfaceObject]
interface SVGPathSegLinetoAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
};

[NoInterfaceObject]
interface SVGPathSegLinetoRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoCubicAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float x1;
  [SetterThrows]
  attribute float y1;
  [SetterThrows]
  attribute float x2;
  [SetterThrows]
  attribute float y2;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoCubicRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float x1;
  [SetterThrows]
  attribute float y1;
  [SetterThrows]
  attribute float x2;
  [SetterThrows]
  attribute float y2;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoQuadraticAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float x1;
  [SetterThrows]
  attribute float y1;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoQuadraticRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float x1;
  [SetterThrows]
  attribute float y1;
};

[NoInterfaceObject]
interface SVGPathSegArcAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float r1;
  [SetterThrows]
  attribute float r2;
  [SetterThrows]
  attribute float angle;
  [SetterThrows]
  attribute boolean largeArcFlag;
  [SetterThrows]
  attribute boolean sweepFlag;
};

[NoInterfaceObject]
interface SVGPathSegArcRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float r1;
  [SetterThrows]
  attribute float r2;
  [SetterThrows]
  attribute float angle;
  [SetterThrows]
  attribute boolean largeArcFlag;
  [SetterThrows]
  attribute boolean sweepFlag;
};

[NoInterfaceObject]
interface SVGPathSegLinetoHorizontalAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
};

[NoInterfaceObject]
interface SVGPathSegLinetoHorizontalRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
};

[NoInterfaceObject]
interface SVGPathSegLinetoVerticalAbs : SVGPathSeg {
  [SetterThrows]
  attribute float y;
};

[NoInterfaceObject]
interface SVGPathSegLinetoVerticalRel : SVGPathSeg {
  [SetterThrows]
  attribute float y;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoCubicSmoothAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float x2;
  [SetterThrows]
  attribute float y2;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoCubicSmoothRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
  [SetterThrows]
  attribute float x2;
  [SetterThrows]
  attribute float y2;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoQuadraticSmoothAbs : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
};

[NoInterfaceObject]
interface SVGPathSegCurvetoQuadraticSmoothRel : SVGPathSeg {
  [SetterThrows]
  attribute float x;
  [SetterThrows]
  attribute float y;
};

