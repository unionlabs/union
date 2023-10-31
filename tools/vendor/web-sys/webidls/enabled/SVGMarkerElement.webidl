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

interface SVGMarkerElement : SVGElement {

  // Marker Unit Types
  const unsigned short SVG_MARKERUNITS_UNKNOWN = 0;
  const unsigned short SVG_MARKERUNITS_USERSPACEONUSE = 1;
  const unsigned short SVG_MARKERUNITS_STROKEWIDTH = 2;

  // Marker Orientation Types
  const unsigned short SVG_MARKER_ORIENT_UNKNOWN = 0;
  const unsigned short SVG_MARKER_ORIENT_AUTO = 1;
  const unsigned short SVG_MARKER_ORIENT_ANGLE = 2;

  [Constant]
  readonly attribute SVGAnimatedLength refX;
  [Constant]
  readonly attribute SVGAnimatedLength refY;
  [Constant]
  readonly attribute SVGAnimatedEnumeration markerUnits;
  [Constant]
  readonly attribute SVGAnimatedLength markerWidth;
  [Constant]
  readonly attribute SVGAnimatedLength markerHeight;
  [Constant]
  readonly attribute SVGAnimatedEnumeration orientType;
  [Constant]
  readonly attribute SVGAnimatedAngle orientAngle;

  undefined setOrientToAuto();
  [Throws]
  undefined setOrientToAngle(SVGAngle angle);
};

SVGMarkerElement includes SVGFitToViewBox;
